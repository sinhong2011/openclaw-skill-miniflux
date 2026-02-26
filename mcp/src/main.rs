mod config;
mod server;

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

    if let Err(e) = server::run(config).await {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }
}
