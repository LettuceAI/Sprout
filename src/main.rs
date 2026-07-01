mod gpu;
mod specs;

use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use clap::Parser;

use crate::specs::SpecsResponse;

#[derive(Parser, Debug)]
#[command(name = "sprout", version, about = "Hardware-info probe for LettuceAI runnability checks")]
struct Args {
    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "Address to bind; use 0.0.0.0 to accept requests from other machines"
    )]
    host: String,
    #[arg(long, default_value_t = 8477, help = "Port to listen on")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();

    let app = Router::new()
        .route("/health", get(health))
        .route("/specs", get(specs));

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("sprout listening on http://{addr}/specs");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn specs() -> Json<SpecsResponse> {
    Json(specs::collect())
}
