# Miniflux MCP Server — Design

## Overview

A Rust CLI MCP server (`openclaw-miniflux-mcp`) that exposes Miniflux RSS reader operations as MCP tools. Distributed as pre-built binaries via GitHub Releases. Includes an OpenClaw skill for agent usage.

## Architecture

Single Rust binary using:
- `rmcp` (server + transport-io) — MCP protocol layer
- `miniflux_api` — Miniflux REST client
- `clap` (derive) — CLI argument parsing
- `tokio` — async runtime
- `reqwest` — HTTP client (already required by miniflux_api)

## Project Structure

```
openclaw-skill-miniflux/
├── skill/SKILL.md
├── mcp/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs             # CLI parsing, config merge, server startup
│   │   ├── config.rs           # Config struct: url, auth, read_only
│   │   ├── server.rs           # MinifluxServer, ServerHandler impl
│   │   └── tools/
│   │       ├── mod.rs
│   │       ├── entries.rs      # get_entries, get_entry, get_feed_entries
│   │       ├── feeds.rs        # get_feeds, get_feed, get_feed_icon, discover_subscription
│   │       ├── categories.rs   # get_categories
│   │       ├── users.rs        # get_current_user, get_user_by_id, get_user_by_name
│   │       ├── system.rs       # healthcheck, export_opml
│   │       └── write.rs        # update_entry_status, toggle_bookmark, refresh_feed
├── docs/plans/
├── .github/workflows/
│   ├── ci.yml
│   ├── release-please.yml
│   └── release.yml
├── release-please-config.json
├── CLAUDE.md
├── CONTRIBUTING.md
├── LICENSE
└── README.md
```

## Tools

### Read Tools (13)

| Tool | Crate method | Description |
|------|-------------|-------------|
| `miniflux_get_entries` | `get_entries` | List/filter entries (status, starred, date range, pagination) |
| `miniflux_get_entry` | `get_entry` | Get single entry by ID |
| `miniflux_get_feed_entries` | `get_feed_entries` | Get entries for a specific feed |
| `miniflux_get_feeds` | `get_feeds` | List all feeds |
| `miniflux_get_feed` | `get_feed` | Get single feed by ID |
| `miniflux_get_feed_icon` | `get_feed_icon` | Get favicon for a feed |
| `miniflux_discover_subscription` | `discover_subscription` | Discover feeds from a URL |
| `miniflux_get_categories` | `get_categories` | List all categories |
| `miniflux_get_current_user` | `get_current_user` | Get current authenticated user |
| `miniflux_get_user_by_id` | `get_user_by_id` | Get user by ID |
| `miniflux_get_user_by_name` | `get_user_by_name` | Get user by username |
| `miniflux_healthcheck` | `healthcheck` | Verify Miniflux connection |
| `miniflux_export_opml` | `export_opml` | Export feeds as OPML |

### Write Tools (3)

| Tool | Crate method | Description |
|------|-------------|-------------|
| `miniflux_update_entry_status` | `update_entries_status` | Mark entries as read/unread |
| `miniflux_toggle_bookmark` | `toggle_bookmark` | Star/unstar an entry |
| `miniflux_refresh_feed` | `refresh_feed_synchronous` | Trigger feed refresh |

## Configuration

### CLI

```
openclaw-miniflux-mcp [OPTIONS]

Options:
  --miniflux-url <URL>        Miniflux instance URL
  --api-token <TOKEN>         API token auth
  --username <USER>           Username auth (requires --password)
  --password <PASS>           Password auth (requires --username)
  --read-only                 Only allow read operations
```

### Resolution Order

CLI flag > env var > default.

| Setting | CLI flag | Env var | Default |
|---------|----------|---------|---------|
| URL | `--miniflux-url` | `MINIFLUX_URL` | (required) |
| Token | `--api-token` | `MINIFLUX_API_TOKEN` | — |
| Username | `--username` | `MINIFLUX_USERNAME` | — |
| Password | `--password` | `MINIFLUX_PASSWORD` | — |
| Read-only | `--read-only` | `MINIFLUX_READ_ONLY=true` | `false` |

### Validation

- URL is always required
- Must provide either token OR username+password (not both, not neither)
- Fail fast with clear error message on startup

## Read-Only Mode

All tools are always registered. When `read_only` is true, write tools return an error:

```rust
if self.config.read_only {
    return Err(McpError::invalid_request(
        "read-only mode: write operations are disabled"
    ));
}
```

This keeps tools discoverable while preventing mutations.

## Distribution

### GitHub Release Binaries

Cross-compiled for 4 targets:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Binary naming: `openclaw-miniflux-mcp-{target}`

### cargo install

```bash
cargo install openclaw-miniflux-mcp
```

### MCP Client Config

```json
{
  "mcpServers": {
    "miniflux": {
      "command": "/path/to/openclaw-miniflux-mcp",
      "args": [],
      "env": {
        "MINIFLUX_URL": "http://localhost:8080",
        "MINIFLUX_API_TOKEN": "<your-token>"
      }
    }
  }
}
```

Read-only with CLI flag:

```json
{
  "mcpServers": {
    "miniflux": {
      "command": "/path/to/openclaw-miniflux-mcp",
      "args": ["--read-only"],
      "env": {
        "MINIFLUX_URL": "http://localhost:8080",
        "MINIFLUX_API_TOKEN": "<your-token>"
      }
    }
  }
}
```

## Skill

`skill/SKILL.md` follows the memos skill pattern:
- Frontmatter with name + description for agent matching
- Prerequisites: binary download + MCP config setup
- Workflow patterns: browsing, searching, triaging, discovery
- Guardrails: default small page sizes, auth error guidance, bulk confirmation

## Testing

- Unit tests: mock miniflux_api client, test tool functions return correct CallToolResult
- Config tests: CLI + env var merging, validation rules
- Read-only tests: write tools return error when read_only=true
- No live integration tests

## CI/CD

- `ci.yml`: cargo fmt + clippy + test on push/PR to main
- `release-please.yml`: automated versioning with release-please (rust release type)
- `release.yml`: cross-compile 4 targets, upload binaries to GitHub release
