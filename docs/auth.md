---
title: "Authentication & Local Security Guide"
description: "Profile discovery, local credential isolation, and permission handling for developers and AI agents using Claude Sessions CLI."
author: "SpaceCorps"
date: "2026-09-25"
---

# Authentication & Local Security Guide

This document outlines how `claude-sessions` interfaces with local Claude desktop accounts, profile resolution, and security considerations.

## 100% Offline & Zero Cloud API Keys

Unlike remote cloud CLI tools, `claude-sessions` operates strictly on your local disk.
- **Zero API Keys Required:** You never need an Anthropic API token, browser session cookie, or secret key.
- **Zero Network Traffic:** The CLI performs zero HTTP calls, telemetry pings, or background cloud syncs.
- **Complete Privacy:** Your code-tab session contents, chat logs, prompts, and code snippets never leave your local workstation.

## Profile Resolution & Detection

The Claude Desktop app manages accounts and organizations using UUID pairs:
`<ACCOUNT_UUID>/<ORGANIZATION_UUID>`

### Auto-Detection
When running `claude-sessions transfer` without `--to`:
1. The CLI inspects the Claude desktop `config.json` on disk to discover the currently signed-in account UUID.
2. It determines the active organization by identifying the organization directory modified most recently by the Claude Desktop process.
3. If auto-detection succeeds, sessions from all other profiles are merged into this active profile.

### Explicit Profile Specification
If the Claude Desktop application is signed out or the profile is ambiguous, provide the destination explicitly:
```bash
claude-sessions transfer --to <ACCOUNT_UUID>/<ORG_UUID>
```
Any unambiguous prefix works (e.g., `--to 1a2b/3c4d`).

## Filesystem Permissions

Ensure your user account has standard read/write permissions to the application data directory:
- **macOS:** `~/Library/Application Support/Claude/`
- **Linux:** `~/.config/Claude/`
- **Windows:** `%APPDATA%\Claude\`

No elevated permissions (`sudo` or Administrator) are required or recommended.
