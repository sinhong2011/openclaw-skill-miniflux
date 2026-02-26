# openclaw-skill-miniflux

## What This Is

A standalone open-source project providing:
1. An MCP server (`openclaw-miniflux-mcp`) for reading and managing a Miniflux RSS instance
2. An OpenClaw skill (`SKILL.md`) that teaches agents how to use the MCP tools

## Architecture

```
openclaw-skill-miniflux/
├── skill/SKILL.md          # OpenClaw skill
├── mcp/                    # Rust MCP server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # CLI (clap) + startup
│       ├── config.rs       # Config: URL, auth, read-only
│       └── server.rs       # MinifluxServer with 16 MCP tools
└── docs/plans/             # Design + implementation plans
```

## Key Files

- `mcp/src/config.rs` — Loads URL, auth, and read-only from CLI args + env vars
- `mcp/src/server.rs` — MinifluxServer struct with all tool methods via rmcp macros
- `mcp/src/main.rs` — Clap CLI parsing, config creation, server startup

## Conventions

- All tool names use `miniflux_` prefix
- Write tools check `read_only` flag before executing
- Uses `rmcp` macros: `#[tool(tool_box)]`, `#[tool(name, description)]`, `#[tool(param)]`
- Results returned as Debug-formatted text via `Content::text()`
- miniflux_api models don't derive Serialize — use `format!("{:#?}", ...)` for output

## How to Test

```bash
cd mcp
cargo test           # Run all tests
cargo clippy         # Lint
cargo fmt --check    # Format check
```

## Common Tasks

- **Add a new tool:** Add `#[tool]` method to `#[tool(tool_box)] impl MinifluxServer` in `server.rs`
- **Change config:** Update `Cli` struct in `main.rs` and `Config` in `config.rs`
- **Update skill docs:** Edit `skill/SKILL.md`
