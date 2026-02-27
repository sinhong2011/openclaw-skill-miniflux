use std::sync::Arc;

use miniflux_api::MinifluxApi;
use reqwest::Client;
use rmcp::{model::*, tool, ServerHandler, ServiceExt};

use crate::config::Config;

use miniflux_api::models::{EntryStatus, OrderBy, OrderDirection};

fn api_err(e: miniflux_api::ApiError) -> rmcp::Error {
    rmcp::Error::internal_error(format!("{e}"), None)
}

fn parse_err(field: &str, value: &str, valid: &str) -> rmcp::Error {
    rmcp::Error::invalid_params(
        format!("Invalid {field} '{value}'. Must be one of: {valid}"),
        None,
    )
}

fn parse_status(s: &str) -> Result<EntryStatus, rmcp::Error> {
    EntryStatus::try_from(s).map_err(|_| parse_err("status", s, "read, unread, removed"))
}

fn parse_order(s: &str) -> Result<OrderBy, rmcp::Error> {
    OrderBy::try_from(s).map_err(|_| {
        parse_err(
            "order",
            s,
            "id, status, published_at, category_title, category_id",
        )
    })
}

fn parse_direction(s: &str) -> Result<OrderDirection, rmcp::Error> {
    OrderDirection::try_from(s).map_err(|_| parse_err("direction", s, "asc, desc"))
}

fn check_write_allowed(read_only: bool) -> Result<(), rmcp::Error> {
    if read_only {
        Err(rmcp::Error::invalid_request(
            "Read-only mode: write operations are disabled. Remove --read-only flag or set MINIFLUX_READ_ONLY=false to enable writes.",
            None,
        ))
    } else {
        Ok(())
    }
}

#[derive(Clone)]
pub struct MinifluxServer {
    api: Arc<MinifluxApi>,
    client: Client,
    read_only: bool,
}

#[tool(tool_box)]
impl MinifluxServer {
    pub fn new(config: &Config) -> Self {
        let api = config.create_api();
        let client = Config::create_client();
        Self {
            api: Arc::new(api),
            client,
            read_only: config.read_only,
        }
    }

    #[tool(
        name = "miniflux_healthcheck",
        description = "Check if the Miniflux instance is reachable and healthy"
    )]
    async fn healthcheck(&self) -> Result<CallToolResult, rmcp::Error> {
        self.api
            .healthcheck(&self.client)
            .await
            .map_err(|e| rmcp::Error::internal_error(format!("Healthcheck failed: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(
            "Miniflux instance is healthy",
        )]))
    }

    #[tool(
        name = "miniflux_get_categories",
        description = "List all feed categories"
    )]
    async fn get_categories(&self) -> Result<CallToolResult, rmcp::Error> {
        let categories = self
            .api
            .get_categories(&self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            categories
        ))]))
    }

    #[tool(
        name = "miniflux_export_opml",
        description = "Export all feeds as OPML XML"
    )]
    async fn export_opml(&self) -> Result<CallToolResult, rmcp::Error> {
        let opml = self.api.export_opml(&self.client).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(opml)]))
    }

    #[tool(name = "miniflux_get_feeds", description = "List all subscribed feeds")]
    async fn get_feeds(&self) -> Result<CallToolResult, rmcp::Error> {
        let feeds = self.api.get_feeds(&self.client).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            feeds
        ))]))
    }

    #[tool(
        name = "miniflux_get_feed",
        description = "Get a single feed by its ID"
    )]
    async fn get_feed(&self, #[tool(param)] id: i64) -> Result<CallToolResult, rmcp::Error> {
        let feed = self.api.get_feed(id, &self.client).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            feed
        ))]))
    }

    #[tool(
        name = "miniflux_get_feed_icon",
        description = "Get the favicon/icon for a feed by feed ID"
    )]
    async fn get_feed_icon(&self, #[tool(param)] id: i64) -> Result<CallToolResult, rmcp::Error> {
        let icon = self
            .api
            .get_feed_icon(id, &self.client)
            .await
            .map_err(api_err)?;
        let json = serde_json::to_string_pretty(&icon)
            .map_err(|e| rmcp::Error::internal_error(format!("{e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        name = "miniflux_discover_subscription",
        description = "Discover RSS/Atom feeds available at a given URL"
    )]
    async fn discover_subscription(
        &self,
        #[tool(param)] url: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        let feed_url = url::Url::parse(&url)
            .map_err(|e| rmcp::Error::invalid_params(format!("Invalid URL: {e}"), None))?;
        let feeds = self
            .api
            .discover_subscription(feed_url, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            feeds
        ))]))
    }

    #[tool(
        name = "miniflux_get_entries",
        description = "List entries with optional filters. Status: 'read', 'unread', 'removed'. Order: 'id', 'status', 'published_at', 'category_title', 'category_id'. Direction: 'asc' or 'desc'."
    )]
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<CallToolResult, rmcp::Error> {
        let status = status.as_deref().map(parse_status).transpose()?;
        let order = order.as_deref().map(parse_order).transpose()?;
        let direction = direction.as_deref().map(parse_direction).transpose()?;
        let entries = self
            .api
            .get_entries(
                status,
                offset,
                limit,
                order,
                direction,
                before,
                after,
                before_entry_id,
                after_entry_id,
                starred,
                &self.client,
            )
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            entries
        ))]))
    }

    #[tool(
        name = "miniflux_get_entry",
        description = "Get a single entry by its ID"
    )]
    async fn get_entry(&self, #[tool(param)] id: i64) -> Result<CallToolResult, rmcp::Error> {
        let entry = self
            .api
            .get_entry(id, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            entry
        ))]))
    }

    #[tool(
        name = "miniflux_get_feed_entries",
        description = "Get entries for a specific feed by feed ID. Accepts same filters as get_entries."
    )]
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<CallToolResult, rmcp::Error> {
        let status = status.as_deref().map(parse_status).transpose()?;
        let order = order.as_deref().map(parse_order).transpose()?;
        let direction = direction.as_deref().map(parse_direction).transpose()?;
        let entries = self
            .api
            .get_feed_entries(
                feed_id,
                status,
                offset,
                limit,
                order,
                direction,
                before,
                after,
                before_entry_id,
                after_entry_id,
                starred,
                &self.client,
            )
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            entries
        ))]))
    }

    #[tool(
        name = "miniflux_get_current_user",
        description = "Get the currently authenticated user's information"
    )]
    async fn get_current_user(&self) -> Result<CallToolResult, rmcp::Error> {
        let user = self
            .api
            .get_current_user(&self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            user
        ))]))
    }

    #[tool(
        name = "miniflux_get_user_by_id",
        description = "Get a user by their numeric ID"
    )]
    async fn get_user_by_id(&self, #[tool(param)] id: i64) -> Result<CallToolResult, rmcp::Error> {
        let user = self
            .api
            .get_user_by_id(id, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            user
        ))]))
    }

    #[tool(
        name = "miniflux_get_user_by_name",
        description = "Get a user by their username"
    )]
    async fn get_user_by_name(
        &self,
        #[tool(param)] username: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        let user = self
            .api
            .get_user_by_name(&username, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            user
        ))]))
    }

    #[tool(
        name = "miniflux_create_category",
        description = "Create a new feed category with the given title"
    )]
    async fn create_category(
        &self,
        #[tool(param)] title: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        check_write_allowed(self.read_only)?;
        let category = self
            .api
            .create_category(&title, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:#?}",
            category
        ))]))
    }

    #[tool(
        name = "miniflux_create_feed",
        description = "Subscribe to a new feed by URL and assign it to a category. Returns the new feed ID."
    )]
    async fn create_feed(
        &self,
        #[tool(param)] feed_url: String,
        #[tool(param)] category_id: i64,
    ) -> Result<CallToolResult, rmcp::Error> {
        check_write_allowed(self.read_only)?;
        let parsed_url = url::Url::parse(&feed_url)
            .map_err(|e| rmcp::Error::invalid_params(format!("Invalid URL: {e}"), None))?;
        let feed_id = self
            .api
            .create_feed(&parsed_url, category_id, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Feed created successfully with ID: {feed_id}"
        ))]))
    }

    #[tool(
        name = "miniflux_update_entry_status",
        description = "Mark one or more entries as read, unread, or removed. Provide a list of entry IDs and a status ('read', 'unread', or 'removed')."
    )]
    async fn update_entry_status(
        &self,
        #[tool(param)] entry_ids: Vec<i64>,
        #[tool(param)] status: String,
    ) -> Result<CallToolResult, rmcp::Error> {
        check_write_allowed(self.read_only)?;
        let status = parse_status(&status)?;
        self.api
            .update_entries_status(entry_ids, status, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Entry status updated successfully",
        )]))
    }

    #[tool(
        name = "miniflux_toggle_bookmark",
        description = "Toggle the bookmark/star status of an entry by its ID"
    )]
    async fn toggle_bookmark(&self, #[tool(param)] id: i64) -> Result<CallToolResult, rmcp::Error> {
        check_write_allowed(self.read_only)?;
        self.api
            .toggle_bookmark(id, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Bookmark toggled successfully",
        )]))
    }

    #[tool(
        name = "miniflux_refresh_feed",
        description = "Trigger a synchronous refresh of a feed by its ID. This fetches new entries from the source."
    )]
    async fn refresh_feed(&self, #[tool(param)] id: i64) -> Result<CallToolResult, rmcp::Error> {
        check_write_allowed(self.read_only)?;
        self.api
            .refresh_feed_synchronous(id, &self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Feed refreshed successfully",
        )]))
    }
}

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
        )
        .unwrap();
        let server = MinifluxServer::new(&config);
        assert!(server.read_only);
    }

    #[test]
    fn test_check_write_allowed_blocks_in_read_only() {
        assert!(check_write_allowed(true).is_err());
    }

    #[test]
    fn test_check_write_allowed_permits_when_not_read_only() {
        assert!(check_write_allowed(false).is_ok());
    }
}

#[tool(tool_box)]
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
    let service = server.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
