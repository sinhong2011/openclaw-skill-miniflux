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
