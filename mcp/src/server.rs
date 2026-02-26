use std::sync::Arc;

use miniflux_api::MinifluxApi;
use reqwest::Client;
use rmcp::{ServerHandler, ServiceExt, model::*, tool};

use crate::config::Config;

fn api_err(e: miniflux_api::ApiError) -> rmcp::Error {
    rmcp::Error::internal_error(format!("{e}"), None)
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
        let opml = self
            .api
            .export_opml(&self.client)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(opml)]))
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
