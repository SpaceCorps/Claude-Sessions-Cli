//! Runs the binary against a throwaway Claude data directory. Never touches the real one.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime};

use serde_json::{Value, json};

const OLD: &str = "aaaa1111/org-old";
const OTHER: &str = "cccc3333/org-other";
const CUR: &str = "bbbb2222/org-cur";
const STALE: &str = "bbbb2222/org-stale";

struct Fixture {
    data: PathBuf,
}

impl Fixture {
    /// Old profile: s1, s2 (archived via the idx), s4, s5. Other profile: an older s1 and s6.
    /// Current profile already has s4 and a tombstone for s5. The signed-in account also has a
    /// stale org that must not be picked as current.
    fn new(name: &str) -> Fixture {
        let data = std::env::temp_dir().join(format!("claude-sessions-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&data);
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("config.json"), r#"{"lastKnownAccountUuid":"bbbb2222","oauth:tokenCache":"x"}"#).unwrap();
        let f = Fixture { data };
        f.session(OLD, "s1", "Engine setup", false, 300);
        f.session(OLD, "s2", "Daily summary", false, 100);
        f.session(OLD, "s4", "Already there", false, 50);
        f.session(OLD, "s5", "Deleted there", false, 40);
        f.write(OLD, "archived-sessions.idx", r#"{"v":1,"archived":["local_s2"]}"#);
        f.session(OTHER, "s1", "Engine setup (old copy)", false, 10);
        f.session(OTHER, "s6", "Archived by flag", true, 20);
        f.write(
            OTHER,
            "local_s7.json",
            r#"{"title":"Gone worktree","worktreePath":"/nonexistent/claude-sessions-wt"}"#,
        );
        f.session(CUR, "s4", "Already there", false, 50);
        f.write(CUR, "deleted_s5", "1790000000000");
        f.write(STALE, "scheduled-tasks.json", "{}");
        let old = SystemTime::now() - Duration::from_secs(86_400);
        fs::File::options()
            .write(true)
            .open(f.profile(STALE).join("scheduled-tasks.json"))
            .unwrap()
            .set_modified(old)
            .unwrap();
        f
    }

    fn profile(&self, profile: &str) -> PathBuf {
        self.data.join("claude-code-sessions").join(profile)
    }

    fn write(&self, profile: &str, file: &str, body: &str) {
        let dir = self.profile(profile);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(file), body).unwrap();
    }

    fn session(&self, profile: &str, id: &str, title: &str, archived: bool, last: i64) {
        let meta = json!({
            "sessionId": format!("local_{id}"),
            "title": title,
            "cwd": "/tmp/project",
            "isArchived": archived,
            "lastActivityAt": 1_790_000_000_000_i64 + last,
        });
        self.write(profile, &format!("local_{id}.json"), &meta.to_string());
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_claude-sessions"))
            .args(args)
            .env("CLAUDE_SESSIONS_DATA_DIR", &self.data)
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> Value {
        let mut args = args.to_vec();
        args.push("--json");
        let out = self.run(&args);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        serde_json::from_slice(&out.stdout).unwrap()
    }

    fn files(&self, profile: &str) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.profile(profile))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.data);
    }
}

fn ids(rows: &Value) -> Vec<&str> {
    rows.as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect()
}

fn reason<'a>(result: &'a Value, id: &str) -> &'a str {
    let rows = result["skipped"].as_array().unwrap();
    rows.iter().find(|r| r["id"] == id).unwrap()["reason"].as_str().unwrap()
}

#[test]
fn list_marks_the_signed_in_accounts_newest_org_as_current() {
    let f = Fixture::new("list");
    let out = f.json(&["list"]);
    assert_eq!(out["current"], CUR);
    let old = out["profiles"].as_array().unwrap().iter().find(|p| p["profile"] == OLD).unwrap();
    assert_eq!(old["sessions"], 4);
    assert_eq!(old["archived"], 1);
    assert_eq!(old["current"], false);
}

#[test]
fn sessions_filters_by_profile_prefix() {
    let f = Fixture::new("sessions");
    let out = f.json(&["sessions", "aaaa"]);
    assert_eq!(ids(&out), ["local_s1", "local_s2", "local_s4", "local_s5"]);
    assert_eq!(out[1]["archived"], true);
    assert_eq!(out[0]["lastActivity"], "2026-09-21T14:13:20Z");
}

#[test]
fn dry_run_writes_nothing() {
    let f = Fixture::new("dry");
    let before = f.files(CUR);
    let out = f.json(&["transfer", "--dry-run"]);
    assert_eq!(out["dryRun"], true);
    assert_eq!(ids(&out["transferred"]), ["local_s1", "local_s2", "local_s6"]);
    assert_eq!(f.files(CUR), before);
}

#[test]
fn transfer_copies_new_sessions_and_skips_the_rest() {
    let f = Fixture::new("transfer");
    let out = f.json(&["transfer"]);
    assert_eq!(out["to"], CUR);
    assert_eq!(ids(&out["transferred"]), ["local_s1", "local_s2", "local_s6"]);
    assert_eq!(reason(&out, "local_s4"), "already in destination");
    assert_eq!(reason(&out, "local_s5"), "deleted in destination");
    assert_eq!(reason(&out, "local_s1"), "newer copy in another profile");
    assert_eq!(reason(&out, "local_s7"), "worktree no longer exists");

    // The newest copy of s1 won, byte for byte.
    let copied = fs::read(f.profile(CUR).join("local_s1.json")).unwrap();
    assert_eq!(copied, fs::read(f.profile(OLD).join("local_s1.json")).unwrap());

    let idx: Value = serde_json::from_slice(&fs::read(f.profile(CUR).join("archived-sessions.idx")).unwrap()).unwrap();
    assert_eq!(idx, json!({ "v": 1, "archived": ["local_s2", "local_s6"] }));

    // Sources are untouched, no temp files are left behind, and a rerun is a no-op.
    assert_eq!(
        f.files(OLD),
        ["archived-sessions.idx", "local_s1.json", "local_s2.json", "local_s4.json", "local_s5.json"]
    );
    assert!(!f.files(CUR).iter().any(|n| n.ends_with(".tmp")));
    let again = f.json(&["transfer"]);
    assert_eq!(again["transferred"], json!([]));
    assert_eq!(again["next"], "Nothing to transfer.");
}

#[test]
fn transfer_honours_from_session_and_skip_archived() {
    let f = Fixture::new("filters");
    let out = f.json(&["transfer", "--from", "aaaa/org-o", "--skip-archived", "--dry-run"]);
    assert_eq!(out["from"], json!([OLD]));
    assert_eq!(ids(&out["transferred"]), ["local_s1"]);
    assert_eq!(reason(&out, "local_s2"), "archived");

    let out = f.json(&["transfer", "--session", "s6"]);
    assert_eq!(ids(&out["transferred"]), ["local_s6"]);
    assert!(f.profile(CUR).join("local_s6.json").exists());
    assert!(!f.profile(CUR).join("local_s1.json").exists());
}

#[test]
fn transfer_to_an_explicit_profile() {
    let f = Fixture::new("to");
    let out = f.json(&["transfer", "--to", "cccc", "--from", "aaaa"]);
    assert_eq!(out["to"], OTHER);
    assert_eq!(ids(&out["transferred"]), ["local_s2", "local_s4", "local_s5"]);
}

#[test]
fn ambiguous_destination_is_invalid_input() {
    let f = Fixture::new("ambiguous");
    let out = f.run(&["transfer", "--to", "bbbb", "--json"]);
    assert_eq!(out.status.code(), Some(6));
    let err: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(err["code"], "invalid_input");
}

#[test]
fn unknown_session_is_not_found() {
    let f = Fixture::new("nosession");
    let out = f.run(&["transfer", "--session", "zzz"]);
    assert_eq!(out.status.code(), Some(4));
}

#[test]
fn missing_signed_in_account_is_no_account() {
    let f = Fixture::new("noaccount");
    fs::write(f.data.join("config.json"), "{}").unwrap();
    let out = f.run(&["transfer"]);
    assert_eq!(out.status.code(), Some(7));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--to"));
}

#[test]
fn missing_data_dir_is_not_found() {
    let out = Command::new(env!("CARGO_BIN_EXE_claude-sessions"))
        .args(["list"])
        .env("CLAUDE_SESSIONS_DATA_DIR", Path::new("/nonexistent/claude-sessions-test"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(4));
}

#[test]
fn agent_readme_prints() {
    let out = Command::new(env!("CARGO_BIN_EXE_claude-sessions")).args(["agent-readme"]).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("claude-sessions transfer"));
}
