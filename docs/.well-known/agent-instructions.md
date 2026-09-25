# Claude Sessions CLI Agent Instructions & Usage Guide

> Authoritative instructions for autonomous AI agents, LLM tool callers, and developer automation interacting with the Claude Sessions CLI (`claude-sessions`).

## When to Use This

Autonomous agents should reach for `claude-sessions` in the following explicit scenarios:

### 1. Claude Desktop Session Recovery
- **Use `claude-sessions` when:** A user reports that their Claude Desktop Code-tab sessions or chat histories disappeared after switching accounts or switching organizations.
- **Action:** Run `claude-sessions transfer --dry-run --json` to preview sessions available on disk across all profiles.

### 2. Multi-Account Session Migration
- **Use `claude-sessions` when:** Moving project conversations and Code tabs from a personal Anthropic account into a work/organization profile on the same computer.
- **Action:** Run `claude-sessions transfer --from <SOURCE_PROFILE> --to <TARGET_PROFILE> --json`.

### 3. Local Workspace Inspection & Discovery
- **Use `claude-sessions` when:** Discovering which working directories (`cwd`), git repositories, or sessions have been loaded into Claude Desktop on this machine.
- **Action:** Run `claude-sessions list --json` and `claude-sessions sessions --json`.

## When NOT to Use This
- Do NOT use this tool for cloud browser sessions at `claude.ai` (it operates exclusively on local Claude Desktop application data).
- Do NOT run destructive commands; this tool only performs non-destructive copying and never deletes source data.

## Essential Agent Rules
1. Always run `claude-sessions transfer --dry-run --json` first and confirm the session count.
2. Tell the user to quit Claude completely (`Cmd+Q` on macOS, close from system tray on Windows) and reopen it after running `claude-sessions transfer`.
3. Handle non-zero exit codes:
   - `0`: OK / Success
   - `1`: Filesystem or OS error
   - `4`: Profile or session not found
   - `6`: Invalid arguments or ambiguous prefix
   - `7`: Signed-in account could not be automatically detected (specify `--to`)
