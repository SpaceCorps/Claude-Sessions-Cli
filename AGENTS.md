# AGENTS.md

Developer and agent notes for `Claude-Sessions-Cli`.

`claude-sessions` copies Claude desktop Code-tab session metadata between account/organization profiles. For the manual an agent reads at runtime, run `claude-sessions agent-readme` (or `--json`). This file is for maintaining the code.

## Quality gates

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Tests set `CLAUDE_SESSIONS_DATA_DIR` to a temp fixture. Never point a test at the real app data directory.

## Layout

```
src/
  main.rs             # --json pre-scan, clap error envelope
  cli.rs              # clap definitions
  error.rs            # ErrorCode (stable exit codes) and Error
  output.rs           # YAML by default, JSON with --json, obj! macro
  store.rs            # the app's on-disk index: profiles, sessions, tombstones, archived idx
  readme.rs           # agent-readme
  commands/
    list.rs           # profiles and counts
    sessions.rs       # session listing
    transfer.rs       # the copy
tests/cli.rs          # end-to-end against a fixture data directory
```

## Invariants

1. **Copy, never move or edit.** Session files are copied byte for byte. The source profile is never written to.
2. **Respect the destination.** Never overwrite a session that already exists there, and never resurrect one it has a `deleted_<uuid>` tombstone for.
3. **Atomic writes.** Copy to `*.tmp`, then rename, so the app never reads half a file.
4. **Preserve unknown fields** in `archived-sessions.idx`; only append to its `archived` array.
5. **Stdout is always valid YAML/JSON**; errors go to stderr as `{error, code, detail?, remediation?}`.

## Exit codes

| Code | Name |
|---|---|
| 1 | error |
| 4 | not_found |
| 6 | invalid_input |
| 7 | no_account |

## App behavior this depends on (observed, app 2.99xx)

- `config.json` `lastKnownAccountUuid` is the signed-in account.
- The app writes into the active org's directory on switch and every turn; the newest directory under the signed-in account is treated as current.
- The app deletes (tombstones) a session on load when its `worktreePath` is gone, so transfer skips those.
- A running app picks up copied files after a restart.
