//! The Claude desktop app's on-disk session index.
//!
//! `<data>/claude-code-sessions/<accountUuid>/<orgUuid>/` holds one `local_<uuid>.json` of
//! metadata per session, `deleted_<uuid>` tombstones, and `archived-sessions.idx`. The sidebar
//! shows only the directory of the signed-in account and its active organization. Transcripts
//! live in `~/.claude/projects` and are shared by every account, so copying the metadata file is
//! enough to bring a session back.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::{Value, json};

use crate::error::{Error, Result};
use crate::obj;

pub const DATA_DIR_ENV: &str = "CLAUDE_SESSIONS_DATA_DIR";
const ARCHIVED_IDX: &str = "archived-sessions.idx";

/// The Claude app's data directory (the parent of `claude-code-sessions`).
pub fn data_dir() -> Result<PathBuf> {
    if let Some(d) = std::env::var_os(DATA_DIR_ENV).filter(|d| !d.is_empty()) {
        return Ok(PathBuf::from(d));
    }
    let base = if cfg!(target_os = "macos") {
        home()?.join("Library/Application Support")
    } else if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from).ok_or_else(|| Error::not_found("APPDATA is not set."))?
    } else {
        match std::env::var_os("XDG_CONFIG_HOME").filter(|d| !d.is_empty()) {
            Some(d) => PathBuf::from(d),
            None => home()?.join(".config"),
        }
    };
    Ok(base.join("Claude"))
}

fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| Error::not_found("Cannot find the home directory."))
}

pub struct Store {
    data: PathBuf,
    root: PathBuf,
}

impl Store {
    pub fn open() -> Result<Store> {
        let data = data_dir()?;
        let root = data.join("claude-code-sessions");
        if !root.is_dir() {
            return Err(Error::not_found(format!("No Claude desktop sessions at {}.", root.display()))
                .fix(format!("{DATA_DIR_ENV}=/path/to/Claude claude-sessions list")));
        }
        Ok(Store { data, root })
    }

    pub fn profiles(&self) -> Result<Vec<Profile>> {
        let mut out = Vec::new();
        for account in subdirs(&self.root)? {
            for org in subdirs(&account)? {
                out.push(Profile { account: file_name(&account), org: file_name(&org), dir: org });
            }
        }
        out.sort_by_key(Profile::id);
        Ok(out)
    }

    /// The account the app last signed in as, from its `config.json`.
    pub fn signed_in_account(&self) -> Option<String> {
        let text = fs::read_to_string(self.data.join("config.json")).ok()?;
        let config: Value = serde_json::from_str(&text).ok()?;
        config.get("lastKnownAccountUuid")?.as_str().map(str::to_string)
    }

    /// The profile the sidebar shows. The app records no active organization on disk, but it
    /// writes into the active one's directory on every switch and every turn, so the signed-in
    /// account's most recently touched organization is it.
    pub fn current(&self) -> Result<Profile> {
        let account = self.signed_in_account().ok_or_else(|| {
            Error::no_account("Cannot tell which account the Claude app is signed in to.")
                .fix("claude-sessions transfer --to <ACCOUNT>/<ORG>")
        })?;
        self.profiles()?.into_iter().filter(|p| p.account == account).max_by_key(Profile::modified).ok_or_else(|| {
            Error::no_account(format!("The signed-in account {account} has no session directory yet."))
                .fix("Open the Code tab in the Claude app once, then retry.")
        })
    }

    /// Profiles matching `ACCOUNT[/ORG]`, where either part may be a UUID prefix.
    pub fn resolve(&self, spec: &str) -> Result<Vec<Profile>> {
        let (account, org) = match spec.split_once('/') {
            Some((a, o)) => (a, Some(o)),
            None => (spec, None),
        };
        if account.is_empty() || org.is_some_and(str::is_empty) {
            return Err(Error::invalid(format!("'{spec}' is not ACCOUNT[/ORG].")));
        }
        let hits: Vec<Profile> = self
            .profiles()?
            .into_iter()
            .filter(|p| p.account.starts_with(account) && org.is_none_or(|o| p.org.starts_with(o)))
            .collect();
        if hits.is_empty() {
            return Err(Error::not_found(format!("No profile matches '{spec}'.")).fix("claude-sessions list"));
        }
        Ok(hits)
    }
}

pub struct Profile {
    pub account: String,
    pub org: String,
    pub dir: PathBuf,
}

impl Profile {
    pub fn id(&self) -> String {
        format!("{}/{}", self.account, self.org)
    }

    fn modified(&self) -> SystemTime {
        let entries = fs::read_dir(&self.dir).into_iter().flatten().flatten();
        entries
            .filter_map(|e| e.metadata().ok()?.modified().ok())
            .max()
            .or_else(|| fs::metadata(&self.dir).ok()?.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    }

    /// Sessions, most recently active first.
    pub fn sessions(&self) -> Result<Vec<Session>> {
        let archived = self.archived_ids();
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            let name = file_name(&path);
            let Some(id) = name.strip_suffix(".json").filter(|id| id.starts_with("local_")) else { continue };
            // A metadata file the app is mid-way through writing still gets copied; only its
            // display fields go missing.
            let meta: Value =
                fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
            out.push(Session {
                archived: meta["isArchived"].as_bool().unwrap_or(false) || archived.contains(id),
                title: meta["title"].as_str().map(str::to_string),
                cwd: meta["cwd"].as_str().map(str::to_string),
                last_activity: meta["lastActivityAt"].as_i64(),
                worktree_gone: meta["worktreePath"].as_str().is_some_and(|w| !Path::new(w).exists()),
                id: id.to_string(),
                path,
            });
        }
        out.sort_by_key(|s| std::cmp::Reverse(s.last_activity));
        Ok(out)
    }

    /// Ids the user deleted here. Copying one back would resurrect it.
    pub fn deleted_ids(&self) -> HashSet<String> {
        let entries = fs::read_dir(&self.dir).into_iter().flatten().flatten();
        entries.filter_map(|e| Some(format!("local_{}", file_name(&e.path()).strip_prefix("deleted_")?))).collect()
    }

    fn archived_ids(&self) -> HashSet<String> {
        let idx = self.read_archived_idx();
        let ids = idx["archived"].as_array().into_iter().flatten();
        ids.filter_map(|v| v.as_str().map(str::to_string)).collect()
    }

    fn read_archived_idx(&self) -> Value {
        fs::read_to_string(self.dir.join(ARCHIVED_IDX))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// Adds ids to `archived-sessions.idx`, keeping whatever else the app stores there.
    pub fn add_archived(&self, ids: &[String]) -> Result<()> {
        let mut idx = self.read_archived_idx();
        if !idx.is_object() {
            idx = json!({ "v": 1 });
        }
        if !idx["archived"].is_array() {
            idx["archived"] = json!([]);
        }
        let list = idx["archived"].as_array_mut().expect("archived is an array");
        for id in ids {
            if !list.iter().any(|v| v.as_str() == Some(id)) {
                list.push(json!(id));
            }
        }
        write_atomic(&self.dir.join(ARCHIVED_IDX), serde_json::to_string(&idx).unwrap_or_default().as_bytes())
    }
}

pub struct Session {
    pub id: String,
    pub path: PathBuf,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub archived: bool,
    pub last_activity: Option<i64>,
    /// The app deletes a worktree session on load once its worktree directory is gone.
    pub worktree_gone: bool,
}

impl Session {
    pub fn to_json(&self, profile: &Profile) -> Value {
        obj! {
            "id" => self.id,
            "title" => self.title,
            "profile" => profile.id(),
            "archived" => self.archived,
            "lastActivity" => self.last_activity.map(iso8601),
            "cwd" => self.cwd,
        }
    }
}

/// Copies through a temp file so the app never reads a half-written session.
pub fn copy_atomic(from: &Path, to: &Path) -> Result<()> {
    let tmp = to.with_extension("tmp");
    fs::copy(from, &tmp)?;
    fs::rename(&tmp, to)?;
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn subdirs(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            out.push(entry.path());
        }
    }
    Ok(out)
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Unix milliseconds as UTC ISO 8601 (Hinnant's civil-from-days), without pulling in chrono.
pub fn iso8601(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    use super::iso8601;

    #[test]
    fn formats_unix_millis() {
        assert_eq!(iso8601(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601(1_000_000_000_000), "2001-09-09T01:46:40Z");
        assert_eq!(iso8601(951_782_400_000), "2000-02-29T00:00:00Z");
    }
}
