# Claude Sessions CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/Claude-Sessions-Cli?color=blue&label=version)](https://github.com/SpaceCorps/Claude-Sessions-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/Claude-Sessions-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Claude-Sessions-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/Claude-Sessions-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Get your Claude desktop Code-tab sessions back after switching accounts or organizations.

The Claude desktop app keeps a separate session list for every account and organization. Sign in with a different account (or switch org) and your sidebar is suddenly empty, even though every session is still on disk. `claude-sessions transfer` copies them into the list the app is showing now.

```bash
claude-sessions transfer --dry-run   # see what would move
claude-sessions transfer             # copy it
```

Then quit Claude completely (Cmd+Q on macOS) and reopen it.

> **Unofficial.** This reads and writes the Claude desktop app's private session index, which can change between app versions. It only ever copies; the source profile is never modified.

---

## Highlights

- ⚡ **Sub-2ms Startup**: Compiled as a native static Rust binary with zero runtime dependencies.
- 🛡️ **Non-Destructive Copying**: Source profiles are never modified, moved, or deleted. Rerunning is always safe.
- 🔒 **100% Offline & Private**: Zero network calls, zero API keys required, and zero telemetry.
- 🤖 **AI Agent Native**: Machine-readable `--json` output, deterministic exit codes, and embedded `agent-readme`.
- 🔍 **Auto-Detection**: Automatically identifies signed-in accounts and active orgs from local config files.

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/Claude-Sessions-Cli --locked
```

### Pre-built Standalone Binaries

Download standalone binary archives directly from the [GitHub Releases](https://github.com/SpaceCorps/Claude-Sessions-Cli/releases/latest) page:

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | [`claude-sessions-v1.0.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Claude-Sessions-Cli/releases/download/v1.0.0/claude-sessions-v1.0.0-aarch64-apple-darwin.tar.gz) |
| **macOS** | Intel (`x86_64`) | [`claude-sessions-v1.0.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Claude-Sessions-Cli/releases/download/v1.0.0/claude-sessions-v1.0.0-x86_64-apple-darwin.tar.gz) |
| **Linux** | x86_64 (musl static) | [`claude-sessions-v1.0.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Claude-Sessions-Cli/releases/download/v1.0.0/claude-sessions-v1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| **Linux** | ARM64 (musl static) | [`claude-sessions-v1.0.0-aarch64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Claude-Sessions-Cli/releases/download/v1.0.0/claude-sessions-v1.0.0-aarch64-unknown-linux-musl.tar.gz) |
| **Windows**| x64 (MSVC) | [`claude-sessions-v1.0.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/Claude-Sessions-Cli/releases/download/v1.0.0/claude-sessions-v1.0.0-x86_64-pc-windows-msvc.zip) |

---

## Commands

| Command | What it does |
|:---|:---|
| `claude-sessions list` | Every account/org profile on this machine, its session count, and which one the app is showing |
| `claude-sessions sessions [ACCOUNT[/ORG]]` | Sessions with id, title, working directory, archived flag and last activity |
| `claude-sessions transfer` | Copy sessions from every other profile into the current one |
| `claude-sessions agent-readme [--json]` | Operating manual for AI agents |

Output is YAML; add `--json` for JSON. Profiles are `ACCOUNT/ORG` UUID pairs, and any unambiguous prefix works (`--from 1a2b/3c4d`).

### `transfer` options

| Flag | Effect |
|:---|:---|
| `--from ACCOUNT[/ORG]` | Only copy from these profiles (repeatable). Default: all others |
| `--to ACCOUNT/ORG` | Copy into this profile. Default: the signed-in account's active org |
| `--session ID` | Only copy these sessions (id prefix, repeatable) |
| `--skip-archived` | Leave archived sessions behind (by default they come over still archived) |
| `--dry-run` | Report without writing |

A session is skipped, with a `reason`, when it is already in the destination, was deleted there, its git worktree no longer exists (the app would delete it on load), or a newer copy exists in another profile. Rerunning is always safe.

---

## How it works

On macOS the app stores its index in `~/Library/Application Support/Claude/claude-code-sessions/<account>/<org>/` (`%APPDATA%\Claude` on Windows, `~/.config/Claude` on Linux):

- `local_<uuid>.json`: one metadata file per session
- `deleted_<uuid>`: a tombstone for a session you deleted
- `archived-sessions.idx`: the archived session ids

Conversation transcripts live in `~/.claude/projects/` and are shared by every account, so copying the metadata file brings a session back intact. The signed-in account comes from the app's `config.json`; its active org is the one the app wrote to most recently.

**Not transferred:** scheduled tasks and organization-level connectors, which belong to the old account.

Set `CLAUDE_SESSIONS_DATA_DIR` to use a data directory other than the default.

---

## Documentation & Links

- 🌐 **Web Documentation:** [https://spacecorps.github.io/Claude-Sessions-Cli/](https://spacecorps.github.io/Claude-Sessions-Cli/)
- 🤖 **Agent Manual (`llms.txt`):** [https://spacecorps.github.io/Claude-Sessions-Cli/llms.txt](https://spacecorps.github.io/Claude-Sessions-Cli/llms.txt)
- 🏢 **SpaceCorps Developer Hub:** [https://spacecorps.github.io](https://spacecorps.github.io)

---

## Development

```bash
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
```

Tests run against a throwaway data directory and never touch the real one. See [AGENTS.md](AGENTS.md).

## License

MIT © SpaceCorps
