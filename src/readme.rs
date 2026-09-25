//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "claude-sessions",
            "version" => env!("CARGO_PKG_VERSION"),
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified (usually a filesystem error), report and stop",
                "4" => "not_found - no Claude data, profile, or matching session; do not retry",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - cannot tell the signed-in account; pass --to",
            },
        });
        return;
    }
    println!("{README}");
}

const RULES: &[&str] = &[
    "Run 'claude-sessions transfer --dry-run' first and show the user what would move.",
    "Transfer copies; it never deletes or edits the source profile.",
    "Sessions already in the destination, or deleted there, are skipped. Rerunning is safe.",
    "After a real transfer the user must quit and reopen the Claude app; do not tell them it is done until then.",
    "Profiles are ACCOUNT/ORG UUID pairs; any unambiguous prefix of either part works.",
    "Use --json when you are going to parse the output.",
];

const README: &str = r#"# claude-sessions - agent operating manual

Copies Claude desktop Code-tab sessions between account/organization profiles. The app shows
only the sessions of the signed-in account and active organization; everything else is still
on disk. Results are YAML on stdout, errors are YAML on stderr, and `--json` switches both to JSON.

## Commands

    claude-sessions list                        # profiles, session counts, which one is current
    claude-sessions sessions [ACCOUNT[/ORG]]    # sessions with id, title, cwd, archived
    claude-sessions transfer --dry-run          # what would be copied into the current profile
    claude-sessions transfer                    # copy every other profile's sessions in
    claude-sessions transfer --from 1a2b/3c4d --session 5e6f7a8b
    claude-sessions transfer --to 9f8e/7d6c --skip-archived

## After a transfer

The app reads its session list only at startup. Tell the user to quit Claude completely
(Cmd+Q on macOS) and reopen it. `appRunning: true` in the output means it is still open.

## Rules

- Dry-run first and show the user the list.
- Transfer copies, never moves. The source profile is untouched.
- Skipped sessions carry a `reason`: already in destination, deleted in destination, worktree
  no longer exists (the app would delete it), archived (with --skip-archived), or newer copy
  in another profile.
- Scheduled tasks and organization connectors are not transferred.

## Exit codes

| Code | Name | What to do |
|---|---|---|
| 0 | ok | |
| 1 | error | Filesystem error. Report and stop. |
| 4 | not_found | No Claude data, profile or session matched. Run `claude-sessions list`. |
| 6 | invalid_input | Fix the arguments. |
| 7 | no_account | Signed-in account unknown. Pass `--to ACCOUNT/ORG`. |

Set CLAUDE_SESSIONS_DATA_DIR to point at a Claude data directory other than the default."#;
