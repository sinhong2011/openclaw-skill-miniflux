# Miniflux MCP Server — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI MCP server that exposes 13 read + 3 write Miniflux API operations as MCP tools, with read-only mode support.

**Architecture:** Single Rust binary using `rmcp` for MCP protocol, `miniflux_api` for the Miniflux REST client, and `clap` for CLI parsing. All 16 tools are methods on a `MinifluxServer` struct via `#[tool_router]`. Read-only mode rejects write calls at runtime.

**Tech Stack:** Rust, rmcp, miniflux_api, clap, tokio, reqwest, serde, schemars

---

### Task 1: Project Scaffold

**Files:**
- Create: `mcp/Cargo.toml`
- Create: `mcp/src/main.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "openclaw-miniflux-mcp"
version = "0.1.0"
edition = "2021"
description = "MCP server for Miniflux RSS reader"
license = "MIT"
repository = "https://github.com/sinhong2011/openclaw-skill-miniflux"

[[bin]]
name = "openclaw-miniflux-mcp"
path = "src/main.rs"

[dependencies]
rmcp = { version = "0.1", features = ["server", "transport-io", "macros"] }
miniflux_api = "0.1"
clap = { version = "4", features = ["derive", "env"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "0.8"
url = "2"

[dev-dependencies]
```

**Step 2: Create minimal main.rs**

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "openclaw-miniflux-mcp", version, about = "MCP server for Miniflux RSS reader")]
struct Cli {
    #[arg(long, env = "MINIFLUX_URL")]
    miniflux_url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    eprintln!("Miniflux MCP server starting with URL: {}", cli.miniflux_url);
}
```

**Step 3: Verify it compiles**

Run: `cd mcp && cargo build 2>&1`
Expected: Compiles successfully (may take a while for first build)

**Step 4: Commit**

```bash
git add mcp/Cargo.toml mcp/src/main.rs
git commit -m "feat: scaffold Rust MCP project with dependencies"
```

---

### Task 2: Config Module

**Files:**
- Create: `mcp/src/config.rs`
- Modify: `mcp/src/main.rs`

**Step 1: Write config tests**

Add to bottom of `mcp/src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_auth_valid() {
        let config = Config::new(
            "http://localhost:8080".into(),
            Some("mytoken".into()),
            None,
            None,
            false,
        );
        assert!(config.is_ok());
    }

    #[test]
    fn test_userpass_auth_valid() {
        let config = Config::new(
            "http://localhost:8080".into(),
            None,
            Some("admin".into()),
            Some("pass".into()),
            false,
        );
        assert!(config.is_ok());
    }

    #[test]
    fn test_no_auth_fails() {
        let config = Config::new(
            "http://localhost:8080".into(),
            None,
            None,
            None,
            false,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_partial_userpass_fails() {
        let config = Config::new(
            "http://localhost:8080".into(),
            None,
            Some("admin".into()),
            None,
            false,
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_read_only_flag() {
        let config = Config::new(
            "http://localhost:8080".into(),
            Some("token".into()),
            None,
            None,
            true,
        ).unwrap();
        assert!(config.read_only);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cd mcp && cargo test 2>&1`
Expected: FAIL — `Config` not defined yet

**Step 3: Implement config.rs**

```rust
use miniflux_api::MinifluxApi;
use reqwest::Client;
use url::Url;

pub enum Auth {
    Token(String),
    UserPass { username: String, password: String },
}

pub struct Config {
    pub url: Url,
    pub auth: Auth,
    pub read_only: bool,
}

impl Config {
    pub fn new(
        url: String,
        api_token: Option<String>,
        username: Option<String>,
        password: Option<String>,
        read_only: bool,
    ) -> Result<Self, String> {
        let url = Url::parse(&url).map_err(|e| format!("Invalid URL: {e}"))?;

        let auth = match (api_token, username, password) {
            (Some(token), _, _) => Auth::Token(token),
            (None, Some(user), Some(pass)) => Auth::UserPass {
                username: user,
                password: pass,
            },
            (None, Some(_), None) | (None, None, Some(_)) => {
                return Err("Both --username and --password are required for user/pass auth".into());
            }
            (None, None, None) => {
                return Err(
                    "Authentication required: provide --api-token or --username + --password".into(),
                );
            }
        };

        Ok(Config {
            url,
            auth,
            read_only,
        })
    }

    pub fn create_api(&self) -> MinifluxApi {
        match &self.auth {
            Auth::Token(token) => MinifluxApi::new_from_token(&self.url, token.clone()),
            Auth::UserPass { username, password } => {
                MinifluxApi::new(&self.url, username.clone(), password.clone())
            }
        }
    }

    pub fn create_client() -> Client {
        Client::new()
    }
}

#[cfg(test)]
mod tests {
    // ... (tests from Step 1)
}
```

**Step 4: Update main.rs to use config**

```rust
mod config;

use clap::Parser;
use config::Config;

#[derive(Parser)]
#[command(name = "openclaw-miniflux-mcp", version, about = "MCP server for Miniflux RSS reader")]
struct Cli {
    /// Miniflux instance URL
    #[arg(long, env = "MINIFLUX_URL")]
    miniflux_url: String,

    /// API token for authentication
    #[arg(long, env = "MINIFLUX_API_TOKEN")]
    api_token: Option<String>,

    /// Username for authentication (requires --password)
    #[arg(long, env = "MINIFLUX_USERNAME")]
    username: Option<String>,

    /// Password for authentication (requires --username)
    #[arg(long, env = "MINIFLUX_PASSWORD")]
    password: Option<String>,

    /// Only allow read operations
    #[arg(long, env = "MINIFLUX_READ_ONLY")]
    read_only: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = Config::new(
        cli.miniflux_url,
        cli.api_token,
        cli.username,
        cli.password,
        cli.read_only,
    )
    .unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    eprintln!(
        "Miniflux MCP server starting (read_only={})",
        config.read_only
    );
}
```

**Step 5: Run tests to verify they pass**

Run: `cd mcp && cargo test 2>&1`
Expected: All 5 tests PASS

**Step 6: Commit**

```bash
git add mcp/src/config.rs mcp/src/main.rs
git commit -m "feat: add config module with CLI args, env vars, and auth validation"
```

---

### Task 3: Server Skeleton + Healthcheck Tool

**Files:**
- Create: `mcp/src/server.rs`
- Modify: `mcp/src/main.rs`

**Step 1: Create server.rs with healthcheck tool**

```rust
use miniflux_api::MinifluxApi;
use reqwest::Client;
use rmcp::{
    ErrorData as McpError,
    model::*,
    tool, tool_router, tool_handler,
    handler::server::tool::ToolRouter,
    service::ServiceExt,
};

use crate::config::Config;

#[derive(Clone)]
pub struct MinifluxServer {
    api: MinifluxApi,
    client: Client,
    read_only: bool,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MinifluxServer {
    pub fn new(config: &Config) -> Self {
        let api = config.create_api();
        let client = Config::create_client();
        Self {
            api,
            client,
            read_only: config.read_only,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(name = "miniflux_healthcheck", description = "Check if the Miniflux instance is reachable and healthy")]
    async fn healthcheck(&self) -> Result<CallToolResult, McpError> {
        self.api
            .healthcheck(&self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("Healthcheck failed: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            "Miniflux instance is healthy",
        )]))
    }
}

#[tool_handler]
impl ServerHandler for MinifluxServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Miniflux RSS reader MCP server. Provides tools to read feeds, entries, categories, and manage read status.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let server = MinifluxServer::new(&config);
    let service = server
        .serve(rmcp::transport::io::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
```

**Step 2: Wire up main.rs**

Replace the `eprintln!` at the end of main with:

```rust
mod config;
mod server;

// ... (Cli struct stays the same)

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = Config::new(
        cli.miniflux_url,
        cli.api_token,
        cli.username,
        cli.password,
        cli.read_only,
    )
    .unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    if let Err(e) = server::run(config).await {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }
}
```

**Step 3: Verify it compiles**

Run: `cd mcp && cargo build 2>&1`
Expected: Compiles successfully

**Step 4: Smoke test — verify healthcheck tool is listed**

Run: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' | cd mcp && cargo run -- --miniflux-url http://localhost:8080 --api-token test 2>/dev/null | head -1`
Expected: JSON response with server info (may fail on actual healthcheck but should initialize)

**Step 5: Commit**

```bash
git add mcp/src/server.rs mcp/src/main.rs
git commit -m "feat: add MCP server skeleton with healthcheck tool"
```

---

### Task 4: Read Tools — Categories & System

**Files:**
- Modify: `mcp/src/server.rs`

**Step 1: Add get_categories, export_opml tools**

Add these methods inside the `#[tool_router] impl MinifluxServer` block:

```rust
    #[tool(name = "miniflux_get_categories", description = "List all feed categories")]
    async fn get_categories(&self) -> Result<CallToolResult, McpError> {
        let categories = self.api
            .get_categories(&self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&categories)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_export_opml", description = "Export all feeds as OPML XML")]
    async fn export_opml(&self) -> Result<CallToolResult, McpError> {
        let opml = self.api
            .export_opml(&self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(opml)]))
    }
```

**Step 2: Verify it compiles**

Run: `cd mcp && cargo build 2>&1`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add mcp/src/server.rs
git commit -m "feat: add get_categories and export_opml tools"
```

---

### Task 5: Read Tools — Feeds

**Files:**
- Modify: `mcp/src/server.rs`

**Step 1: Add feed tools**

Add inside `#[tool_router] impl MinifluxServer`:

```rust
    #[tool(name = "miniflux_get_feeds", description = "List all subscribed feeds")]
    async fn get_feeds(&self) -> Result<CallToolResult, McpError> {
        let feeds = self.api
            .get_feeds(&self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&feeds)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_get_feed", description = "Get a single feed by its ID")]
    async fn get_feed(&self, #[tool(param)] id: i64) -> Result<CallToolResult, McpError> {
        let feed = self.api
            .get_feed(id, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&feed)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_get_feed_icon", description = "Get the favicon/icon for a feed by feed ID")]
    async fn get_feed_icon(&self, #[tool(param)] id: i64) -> Result<CallToolResult, McpError> {
        let icon = self.api
            .get_feed_icon(id, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&icon)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_discover_subscription", description = "Discover RSS/Atom feeds available at a given URL")]
    async fn discover_subscription(&self, #[tool(param)] url: String) -> Result<CallToolResult, McpError> {
        let feed_url = url::Url::parse(&url)
            .map_err(|e| McpError::invalid_params(format!("Invalid URL: {e}"), None))?;
        let feeds = self.api
            .discover_subscription(feed_url, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&feeds)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
```

**Step 2: Verify it compiles**

Run: `cd mcp && cargo build 2>&1`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add mcp/src/server.rs
git commit -m "feat: add feed tools (get_feeds, get_feed, get_feed_icon, discover_subscription)"
```

---

### Task 6: Read Tools — Entries

**Files:**
- Modify: `mcp/src/server.rs`

The entries tools are the most complex because `get_entries` and `get_feed_entries` accept many optional filter parameters.

**Step 1: Add entry tools**

Add inside `#[tool_router] impl MinifluxServer`:

```rust
    #[tool(name = "miniflux_get_entries", description = "List entries with optional filters. All parameters are optional. Status can be 'read', 'unread', or 'removed'. Order is 'id', 'status', 'published_at', 'category_title', 'category_id'. Direction is 'asc' or 'desc'.")]
    async fn get_entries(
        &self,
        #[tool(param)] status: Option<String>,
        #[tool(param)] offset: Option<i64>,
        #[tool(param)] limit: Option<i64>,
        #[tool(param)] order: Option<String>,
        #[tool(param)] direction: Option<String>,
        #[tool(param)] before: Option<i64>,
        #[tool(param)] after: Option<i64>,
        #[tool(param)] before_entry_id: Option<i64>,
        #[tool(param)] after_entry_id: Option<i64>,
        #[tool(param)] starred: Option<bool>,
    ) -> Result<CallToolResult, McpError> {
        let status = status.as_deref().map(parse_entry_status).transpose()?;
        let entries = self.api
            .get_entries(
                status, offset, limit,
                order.as_deref(), direction.as_deref(),
                before, after, before_entry_id, after_entry_id, starred,
                &self.client,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_get_entry", description = "Get a single entry by its ID")]
    async fn get_entry(&self, #[tool(param)] id: i64) -> Result<CallToolResult, McpError> {
        let entry = self.api
            .get_entry(id, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&entry)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_get_feed_entries", description = "Get entries for a specific feed by feed ID. Accepts same filters as get_entries.")]
    async fn get_feed_entries(
        &self,
        #[tool(param)] feed_id: i64,
        #[tool(param)] status: Option<String>,
        #[tool(param)] offset: Option<i64>,
        #[tool(param)] limit: Option<i64>,
        #[tool(param)] order: Option<String>,
        #[tool(param)] direction: Option<String>,
        #[tool(param)] before: Option<i64>,
        #[tool(param)] after: Option<i64>,
        #[tool(param)] before_entry_id: Option<i64>,
        #[tool(param)] after_entry_id: Option<i64>,
        #[tool(param)] starred: Option<bool>,
    ) -> Result<CallToolResult, McpError> {
        let status = status.as_deref().map(parse_entry_status).transpose()?;
        let entries = self.api
            .get_feed_entries(
                feed_id,
                status, offset, limit,
                order.as_deref(), direction.as_deref(),
                before, after, before_entry_id, after_entry_id, starred,
                &self.client,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
```

**Step 2: Add helper function** (outside the impl block, at module level in server.rs):

```rust
fn parse_entry_status(s: &str) -> Result<miniflux_api::models::EntryStatus, McpError> {
    match s {
        "read" => Ok(miniflux_api::models::EntryStatus::Read),
        "unread" => Ok(miniflux_api::models::EntryStatus::Unread),
        "removed" => Ok(miniflux_api::models::EntryStatus::Removed),
        _ => Err(McpError::invalid_params(
            format!("Invalid status '{s}'. Must be 'read', 'unread', or 'removed'"),
            None,
        )),
    }
}
```

> **Note:** The exact `EntryStatus` variant names depend on the crate. Check `miniflux_api::models::EntryStatus` and adjust if needed.

**Step 3: Verify it compiles**

Run: `cd mcp && cargo build 2>&1`
Expected: Compiles (adjust EntryStatus variants and method signatures if needed based on actual crate API)

**Step 4: Commit**

```bash
git add mcp/src/server.rs
git commit -m "feat: add entry tools (get_entries, get_entry, get_feed_entries)"
```

---

### Task 7: Read Tools — Users

**Files:**
- Modify: `mcp/src/server.rs`

**Step 1: Add user tools**

Add inside `#[tool_router] impl MinifluxServer`:

```rust
    #[tool(name = "miniflux_get_current_user", description = "Get the currently authenticated user's information")]
    async fn get_current_user(&self) -> Result<CallToolResult, McpError> {
        let user = self.api
            .get_current_user(&self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&user)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_get_user_by_id", description = "Get a user by their numeric ID")]
    async fn get_user_by_id(&self, #[tool(param)] id: i64) -> Result<CallToolResult, McpError> {
        let user = self.api
            .get_user_by_id(id, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&user)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(name = "miniflux_get_user_by_name", description = "Get a user by their username")]
    async fn get_user_by_name(&self, #[tool(param)] username: String) -> Result<CallToolResult, McpError> {
        let user = self.api
            .get_user_by_name(&username, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let json = serde_json::to_string_pretty(&user)
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
```

**Step 2: Verify it compiles**

Run: `cd mcp && cargo build 2>&1`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add mcp/src/server.rs
git commit -m "feat: add user tools (get_current_user, get_user_by_id, get_user_by_name)"
```

---

### Task 8: Write Tools + Read-Only Guard

**Files:**
- Modify: `mcp/src/server.rs`

**Step 1: Write read-only test**

Add to `mcp/src/server.rs` (or a separate test file):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_read_only_config_propagates() {
        let config = Config::new(
            "http://localhost:8080".into(),
            Some("token".into()),
            None,
            None,
            true,
        ).unwrap();
        let server = MinifluxServer::new(&config);
        assert!(server.read_only);
    }
}
```

**Step 2: Add read-only guard helper**

Add at module level in `server.rs`:

```rust
fn check_write_allowed(read_only: bool) -> Result<(), McpError> {
    if read_only {
        Err(McpError::invalid_request(
            "Read-only mode: write operations are disabled. Remove --read-only flag or set MINIFLUX_READ_ONLY=false to enable writes.",
            None,
        ))
    } else {
        Ok(())
    }
}
```

**Step 3: Add write tools**

Add inside `#[tool_router] impl MinifluxServer`:

```rust
    #[tool(name = "miniflux_update_entry_status", description = "Mark one or more entries as read, unread, or removed. Provide a list of entry IDs and a status ('read', 'unread', or 'removed').")]
    async fn update_entry_status(
        &self,
        #[tool(param)] entry_ids: Vec<i64>,
        #[tool(param)] status: String,
    ) -> Result<CallToolResult, McpError> {
        check_write_allowed(self.read_only)?;
        let status = parse_entry_status(&status)?;
        self.api
            .update_entries_status(entry_ids, status, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            "Entry status updated successfully",
        )]))
    }

    #[tool(name = "miniflux_toggle_bookmark", description = "Toggle the bookmark/star status of an entry by its ID")]
    async fn toggle_bookmark(&self, #[tool(param)] id: i64) -> Result<CallToolResult, McpError> {
        check_write_allowed(self.read_only)?;
        self.api
            .toggle_bookmark(id, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            "Bookmark toggled successfully",
        )]))
    }

    #[tool(name = "miniflux_refresh_feed", description = "Trigger a synchronous refresh of a feed by its ID. This fetches new entries from the source.")]
    async fn refresh_feed(&self, #[tool(param)] id: i64) -> Result<CallToolResult, McpError> {
        check_write_allowed(self.read_only)?;
        self.api
            .refresh_feed_synchronous(id, &self.client)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            "Feed refreshed successfully",
        )]))
    }
```

**Step 4: Verify it compiles and tests pass**

Run: `cd mcp && cargo test 2>&1`
Expected: All tests pass

**Step 5: Commit**

```bash
git add mcp/src/server.rs
git commit -m "feat: add write tools with read-only guard (update_entry_status, toggle_bookmark, refresh_feed)"
```

---

### Task 9: Skill

**Files:**
- Create: `skill/SKILL.md`

**Step 1: Write the skill file**

```markdown
---
name: miniflux
description: >
  Read RSS feeds and entries from a Miniflux instance. Handles requests like
  "show my unread articles", "list my feeds", "what feeds do I have in category X",
  "mark these entries as read", or "bookmark this article".
  Uses openclaw-miniflux-mcp for all operations.
---

# Miniflux

## What it does

Provides read access to a Miniflux RSS reader instance through 13 read tools
and 3 optional write tools. Agents can browse feeds, search entries by status
or date, read specific articles, check categories, and (if not in read-only
mode) mark entries as read and toggle bookmarks.

## Inputs needed

- For listing entries: status, date range, starred, pagination (all optional)
- For feed-specific queries: feed ID
- For single items: entry ID, feed ID, or user ID
- For discovery: a URL to scan for feeds
- For writes: entry IDs + status, or entry ID for bookmark toggle

## Prerequisites

### `openclaw-miniflux-mcp` binary

Download the latest binary for your platform from
[GitHub Releases](https://github.com/sinhong2011/openclaw-skill-miniflux/releases):

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

### MCP server configuration

Add the MCP server to your client configuration:

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

Tell the user to:
1. Replace the binary path with wherever they downloaded/installed it
2. Replace `MINIFLUX_URL` with their Miniflux instance URL
3. Get an API token from Miniflux: **Settings > API Keys > Create a new API key**
4. Restart their MCP client after saving

## Workflow

### Browsing feeds

1. Call `miniflux_get_feeds` to see all subscriptions
2. Call `miniflux_get_feed_entries` with a feed ID to see its entries
3. Call `miniflux_get_entry` to read a specific article

### Searching entries

Call `miniflux_get_entries` with filters:
- `status`: `"unread"`, `"read"`, or `"removed"`
- `starred`: `true` for bookmarked entries
- `after` / `before`: Unix timestamps for date ranges
- `limit`: Number of results (default varies, recommend 20)
- `order`: `"published_at"` and `direction`: `"desc"` for newest first

### Triaging unread articles

1. Call `miniflux_get_entries` with `status: "unread"`, `limit: 20`
2. Read interesting entries with `miniflux_get_entry`
3. Mark reviewed entries as read: `miniflux_update_entry_status` with `status: "read"`
4. Bookmark important ones: `miniflux_toggle_bookmark`

### Discovering new feeds

1. Call `miniflux_discover_subscription` with a website URL
2. Present discovered feeds to the user
3. User can subscribe manually in Miniflux UI

### Checking categories

Call `miniflux_get_categories` to list all feed categories.

## Guardrails

- Default to small page sizes (limit=20) to avoid overwhelming responses
- On 401/403 errors, tell the user to check their API token or credentials
- On connection errors, tell the user to verify their MINIFLUX_URL
- Confirm with the user before marking large batches of entries as read
- In read-only mode, explain the limitation clearly when a write is attempted
- When listing returns empty results, suggest checking filters or confirming the instance has data
```

**Step 2: Commit**

```bash
git add skill/SKILL.md
git commit -m "feat: add OpenClaw skill for Miniflux MCP setup and usage"
```

---

### Task 10: CI Workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Create CI workflow**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: mcp
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: mcp
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

**Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add CI workflow with fmt, clippy, and test"
```

---

### Task 11: Release Workflows

**Files:**
- Create: `.github/workflows/release-please.yml`
- Create: `.github/workflows/release.yml`
- Create: `release-please-config.json`
- Create: `.release-please-manifest.json`

**Step 1: Create release-please config**

`release-please-config.json`:

```json
{
  "$schema": "https://raw.githubusercontent.com/googleapis/release-please/main/schemas/config.json",
  "packages": {
    ".": {
      "release-type": "rust",
      "package-name": "openclaw-miniflux-mcp",
      "component": "openclaw-miniflux-mcp",
      "extra-files": [
        "mcp/Cargo.toml"
      ],
      "bump-minor-pre-major": true,
      "bump-patch-for-minor-pre-major": true
    }
  },
  "include-component-in-tag": false,
  "tag-separator": ""
}
```

`.release-please-manifest.json`:

```json
{
  ".": "0.1.0"
}
```

**Step 2: Create release-please workflow**

`.github/workflows/release-please.yml`:

```yaml
name: Release Please

on:
  push:
    branches: [main]

permissions:
  contents: write
  pull-requests: write

jobs:
  release-please:
    runs-on: ubuntu-latest
    outputs:
      release_created: ${{ steps.release.outputs.release_created }}
      tag_name: ${{ steps.release.outputs.tag_name }}
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          config-file: release-please-config.json
          manifest-file: .release-please-manifest.json
          token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
```

**Step 3: Create binary release workflow**

`.github/workflows/release.yml`:

```yaml
name: Release Binaries

on:
  release:
    types: [published]

permissions:
  contents: write

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
    runs-on: ${{ matrix.os }}
    defaults:
      run:
        working-directory: mcp
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross-compilation tools
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-aarch64-linux-gnu
          echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> $GITHUB_ENV
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: mcp
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Rename binary
        run: |
          cp target/${{ matrix.target }}/release/openclaw-miniflux-mcp \
             openclaw-miniflux-mcp-${{ matrix.target }}
      - name: Upload to release
        uses: softprops/action-gh-release@v2
        with:
          files: mcp/openclaw-miniflux-mcp-${{ matrix.target }}

  publish-skill:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20.x
      - run: npx clawhub@latest login --token "$CLAWHUB_TOKEN" --no-browser
        env:
          CLAWHUB_TOKEN: ${{ secrets.CLAWHUB_TOKEN }}
      - run: >
          npx clawhub@latest publish ./skill
          --slug miniflux
          --name "Miniflux"
          --version "${RELEASE_TAG#v}"
          --changelog "Release ${RELEASE_TAG#v}"
          --tags latest
          --no-input
        env:
          RELEASE_TAG: ${{ github.event.release.tag_name }}
```

**Step 4: Commit**

```bash
git add release-please-config.json .release-please-manifest.json \
  .github/workflows/release-please.yml .github/workflows/release.yml
git commit -m "ci: add release-please and binary release workflows"
```

---

### Task 12: Project Documentation

**Files:**
- Create: `CLAUDE.md`
- Create: `CONTRIBUTING.md`
- Create: `LICENSE`
- Create: `README.md`
- Create: `.gitignore`

**Step 1: Create CLAUDE.md**

```markdown
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
- Uses `rmcp` macros: `#[tool_router]`, `#[tool]`, `#[tool_handler]`
- Results returned as JSON text via `Content::text()`

## How to Test

```bash
cd mcp
cargo test           # Run all tests
cargo clippy         # Lint
cargo fmt --check    # Format check
```

## Common Tasks

- **Add a new tool:** Add `#[tool]` method to `#[tool_router] impl MinifluxServer` in `server.rs`
- **Change config:** Update `Cli` struct in `main.rs` and `Config` in `config.rs`
- **Update skill docs:** Edit `skill/SKILL.md`
```

**Step 2: Create CONTRIBUTING.md**

```markdown
# Contributing

Thanks for your interest in contributing!

## Development Setup

```bash
cd mcp
cargo build         # Build
cargo test          # Run tests
cargo clippy        # Lint
cargo fmt           # Format
```

## Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/) with [Release Please](https://github.com/googleapis/release-please) for automated releases.

```
feat: add new feature        → minor version bump
fix: fix a bug               → patch version bump
feat!: breaking change       → major version bump (post-1.0)
chore: maintenance tasks     → no release
docs: documentation only     → no release
```

## Pull Requests

1. Fork the repo and create a branch from `main`
2. Write tests for new functionality
3. Ensure `cargo test`, `cargo clippy`, and `cargo fmt --check` all pass
4. Use conventional commit messages
5. Open a PR against `main`
```

**Step 3: Create LICENSE (MIT)**

Use standard MIT license text with `sinhong2011` as copyright holder.

**Step 4: Create README.md**

Write README following the memos README pattern: badges, quick start with binary download, MCP config examples (token + user/pass + read-only), tool table, configuration table, development section.

**Step 5: Create .gitignore**

```
/mcp/target/
```

**Step 6: Commit**

```bash
git add CLAUDE.md CONTRIBUTING.md LICENSE README.md .gitignore
git commit -m "docs: add project documentation (README, CLAUDE.md, CONTRIBUTING, LICENSE)"
```

---

### Task 13: Final Verification

**Step 1: Run full check suite**

```bash
cd mcp
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Expected: All pass, release binary builds

**Step 2: Verify binary runs**

```bash
./mcp/target/release/openclaw-miniflux-mcp --help
```

Expected: Shows CLI help with all options

**Step 3: Verify binary fails gracefully without auth**

```bash
./mcp/target/release/openclaw-miniflux-mcp --miniflux-url http://localhost:8080 2>&1
```

Expected: Error message about missing authentication

**Step 4: Final commit if any fixes needed**
