---
title: "Claude Sessions CLI"
description: "A blazing fast native command-line tool and agent interface for recovering and migrating Claude desktop Code-tab sessions across accounts and organizations. Built in pure Rust."
author: "SpaceCorps"
date: "2026-09-25"
canonical: "https://spacecorps.github.io/Claude-Sessions-Cli/index.md"
---

# Claude Sessions CLI

A blazing fast native command-line tool and agent interface for transferring and managing Claude desktop Code-tab sessions across multiple accounts and organizations. Built in pure Rust for software engineers and autonomous AI workflows.

## Quickstart

```bash
# Preview sessions that would be migrated into your active profile
claude-sessions transfer --dry-run

# Execute migration (safe, non-destructive copy)
claude-sessions transfer

# Quit Claude completely (Cmd+Q on macOS) and reopen to reload the sidebar
```

## Features

- **Blazing Fast Native Rust**: Sub-millisecond startup times with zero runtime dependencies.
- **Non-Destructive Copying**: Source profiles are never modified or deleted; rerun safely anytime.
- **AI Agent Native**: Structured JSON output (`--json`), deterministic exit codes, and embedded `agent-readme`.
- **Automatic Account Detection**: Detects signed-in accounts and active orgs directly from desktop config files.
- **100% Offline & Private**: Zero network calls, zero API keys required, and zero telemetry.

## Commands Overview

| Command | Description |
|:---|:---|
| `claude-sessions list` | Discover all account/org profiles and session counts on this machine |
| `claude-sessions sessions [ACCOUNT[/ORG]]` | List sessions with IDs, titles, working directory paths, and archive flags |
| `claude-sessions transfer` | Copy sessions from other profiles into the current active profile |
| `claude-sessions agent-readme [--json]` | Output complete agent operating manual |

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Claude-Sessions-Cli/llms.txt)
- [Comprehensive Agent Manual](https://spacecorps.github.io/Claude-Sessions-Cli/llms-full.txt)
- [Authentication & Security Guide](https://spacecorps.github.io/Claude-Sessions-Cli/auth.md)
- [Pricing & Licensing](https://spacecorps.github.io/Claude-Sessions-Cli/pricing.md)
- [GitHub Repository](https://github.com/SpaceCorps/Claude-Sessions-Cli)
