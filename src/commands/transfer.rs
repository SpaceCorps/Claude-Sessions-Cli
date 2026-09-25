//! Copies session metadata files into the destination profile. Copy, never move: the source
//! profile keeps working if the user signs back in to it.

use std::collections::HashSet;
use std::process::Command;

use serde_json::Value;

use crate::cli::TransferArgs;
use crate::error::{Error, Result};
use crate::store::{Profile, Session, Store, copy_atomic};
use crate::{obj, output};

pub fn run(args: &TransferArgs) -> Result<()> {
    let store = Store::open()?;
    let dest = match &args.to {
        Some(spec) => single(store.resolve(spec)?, spec)?,
        None => store.current()?,
    };

    let mut sources = Vec::new();
    if args.from.is_empty() {
        sources = store.profiles()?;
    } else {
        for spec in &args.from {
            for p in store.resolve(spec)? {
                if !sources.iter().any(|s: &Profile| s.id() == p.id()) {
                    sources.push(p);
                }
            }
        }
    }
    sources.retain(|p| p.id() != dest.id());
    if sources.is_empty() {
        return Err(Error::not_found(format!("No profile other than {} to transfer from.", dest.id()))
            .fix("claude-sessions list"));
    }

    // Newest first, so when two profiles hold the same session the freshest copy wins.
    let mut candidates: Vec<(&Profile, Session)> = Vec::new();
    for profile in &sources {
        candidates.extend(profile.sessions()?.into_iter().map(|s| (profile, s)));
    }
    candidates.sort_by_key(|(_, s)| std::cmp::Reverse(s.last_activity));
    if !args.sessions.is_empty() {
        candidates.retain(|(_, s)| args.sessions.iter().any(|want| matches(&s.id, want)));
        if candidates.is_empty() {
            return Err(Error::not_found("No session in the source profiles matches --session.")
                .fix("claude-sessions sessions"));
        }
    }

    let existing: HashSet<String> = dest.sessions()?.into_iter().map(|s| s.id).collect();
    let deleted = dest.deleted_ids();
    let mut seen = HashSet::new();
    let (mut transferred, mut skipped, mut archived) = (Vec::new(), Vec::new(), Vec::new());
    for (profile, session) in candidates {
        let reason = if existing.contains(&session.id) {
            Some("already in destination")
        } else if deleted.contains(&session.id) {
            Some("deleted in destination")
        } else if session.worktree_gone {
            Some("worktree no longer exists")
        } else if args.skip_archived && session.archived {
            Some("archived")
        } else if !seen.insert(session.id.clone()) {
            Some("newer copy in another profile")
        } else {
            None
        };
        let mut row = session.to_json(profile);
        if let Some(reason) = reason {
            row["reason"] = reason.into();
            skipped.push(row);
            continue;
        }
        if !args.dry_run {
            copy_atomic(&session.path, &dest.dir.join(format!("{}.json", session.id)))?;
        }
        if session.archived {
            archived.push(session.id.clone());
        }
        transferred.push(row);
    }
    if !args.dry_run && !archived.is_empty() {
        dest.add_archived(&archived)?;
    }

    let next = if args.dry_run {
        "Dry run: nothing was written."
    } else if transferred.is_empty() {
        "Nothing to transfer."
    } else {
        "Quit the Claude app completely and reopen it; it reads the session list only at startup."
    };
    let mut result = obj! {
        "status" => "ok",
        "dryRun" => args.dry_run,
        "to" => dest.id(),
        "from" => sources.iter().map(Profile::id).collect::<Vec<_>>(),
        "transferred" => Value::Array(transferred),
        "skipped" => Value::Array(skipped),
        "next" => next,
    };
    if let Some(running) = app_running() {
        result["appRunning"] = running.into();
    }
    output::write(&result);
    Ok(())
}

fn single(mut hits: Vec<Profile>, spec: &str) -> Result<Profile> {
    if hits.len() == 1 {
        return Ok(hits.remove(0));
    }
    Err(Error::invalid(format!("'{spec}' matches {} profiles; --to needs exactly one.", hits.len()))
        .detail(hits.iter().map(Profile::id).collect::<Vec<_>>().join("\n"))
        .fix("claude-sessions transfer --to <ACCOUNT>/<ORG>"))
}

/// `want` may be a full id or a prefix, with or without the `local_` prefix.
fn matches(id: &str, want: &str) -> bool {
    let want = want.strip_prefix("local_").unwrap_or(want);
    !want.is_empty() && id.strip_prefix("local_").unwrap_or(id).starts_with(want)
}

/// Whether the desktop app is running, where that is cheap to tell. Only reported: the app
/// picks up copied sessions on its next start either way.
fn app_running() -> Option<bool> {
    if cfg!(target_os = "macos") {
        // ps rather than pgrep: pgrep sees nothing inside the macOS sandbox Claude Code runs in.
        let out = Command::new("ps").args(["-axo", "comm="]).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).lines().any(|l| l.ends_with(".app/Contents/MacOS/Claude")))
    } else if cfg!(windows) {
        let out = Command::new("tasklist").args(["/FI", "IMAGENAME eq Claude.exe", "/NH"]).output().ok()?;
        Some(String::from_utf8_lossy(&out.stdout).contains("Claude.exe"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn session_ids_match_by_prefix() {
        assert!(matches("local_16642da5-342b", "16642da5"));
        assert!(matches("local_16642da5-342b", "local_1664"));
        assert!(!matches("local_16642da5-342b", "342b"));
        assert!(!matches("local_16642da5-342b", "local_"));
    }
}
