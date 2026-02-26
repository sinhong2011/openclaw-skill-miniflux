# openclaw-miniflux-mcp

[![CI](https://github.com/sinhong2011/openclaw-skill-miniflux/actions/workflows/ci.yml/badge.svg)](https://github.com/sinhong2011/openclaw-skill-miniflux/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An MCP server for [Miniflux](https://miniflux.app/) RSS reader. Exposes 13 read tools and 3 write tools for browsing feeds, reading entries, and managing read status.

Includes an [OpenClaw](https://openclaw.dev/) skill that teaches agents how to use the tools.

## Quick Start

### 1. Download the binary

Grab the latest release for your platform from [GitHub Releases](https://github.com/sinhong2011/openclaw-skill-miniflux/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `openclaw-miniflux-mcp-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `openclaw-miniflux-mcp-aarch64-unknown-linux-gnu` |
| macOS x86_64 | `openclaw-miniflux-mcp-x86_64-apple-darwin` |
| macOS ARM64 | `openclaw-miniflux-mcp-aarch64-apple-darwin` |

Or install via Cargo:

```bash
cargo install openclaw-miniflux-mcp
```

### 2. Configure MCP client

**With API token (recommended):**

```json
{
  "mcpServers": {
    "miniflux": {
      "command": "/path/to/openclaw-miniflux-mcp",
      "args": [],
      "env": {
        "MINIFLUX_URL": "http://localhost:8080",
        "MINIFLUX_API_TOKEN": "<your-api-token>"
      }
    }
  }
}
```

**With username/password:**

```json
{
  "mcpServers": {
    "miniflux": {
      "command": "/path/to/openclaw-miniflux-mcp",
      "args": [],
      "env": {
        "MINIFLUX_URL": "http://localhost:8080",
        "MINIFLUX_USERNAME": "<username>",
        "MINIFLUX_PASSWORD": "<password>"
      }
    }
  }
}
```

**Read-only mode** (disables write tools):

```json
{
  "mcpServers": {
    "miniflux": {
      "command": "/path/to/openclaw-miniflux-mcp",
      "args": ["--read-only"],
      "env": {
        "MINIFLUX_URL": "http://localhost:8080",
        "MINIFLUX_API_TOKEN": "<your-api-token>"
      }
    }
  }
}
```

Get an API token from Miniflux: **Settings > API Keys > Create a new API key**

## Tools

### Read Tools (13)

| Tool | Description |
|------|-------------|
| `miniflux_healthcheck` | Check if the Miniflux instance is reachable |
| `miniflux_get_feeds` | List all subscribed feeds |
| `miniflux_get_feed` | Get a single feed by ID |
| `miniflux_get_feed_icon` | Get favicon for a feed |
| `miniflux_discover_subscription` | Discover feeds at a URL |
| `miniflux_get_entries` | List/filter entries (status, starred, date range, pagination) |
| `miniflux_get_entry` | Get a single entry by ID |
| `miniflux_get_feed_entries` | Get entries for a specific feed |
| `miniflux_get_categories` | List all categories |
| `miniflux_get_current_user` | Get current authenticated user |
| `miniflux_get_user_by_id` | Get user by ID |
| `miniflux_get_user_by_name` | Get user by username |
| `miniflux_export_opml` | Export feeds as OPML |

### Write Tools (3)

| Tool | Description |
|------|-------------|
| `miniflux_update_entry_status` | Mark entries as read/unread/removed |
| `miniflux_toggle_bookmark` | Star/unstar an entry |
| `miniflux_refresh_feed` | Trigger feed refresh |

## Configuration

| Setting | CLI flag | Env var | Default |
|---------|----------|---------|---------|
| URL | `--miniflux-url` | `MINIFLUX_URL` | (required) |
| Token | `--api-token` | `MINIFLUX_API_TOKEN` | — |
| Username | `--username` | `MINIFLUX_USERNAME` | — |
| Password | `--password` | `MINIFLUX_PASSWORD` | — |
| Read-only | `--read-only` | `MINIFLUX_READ_ONLY` | `false` |

## Development

```bash
cd mcp
cargo build         # Build
cargo test          # Run tests
cargo clippy        # Lint
cargo fmt --check   # Format check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

MIT
