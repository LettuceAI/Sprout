mod auth;
mod config;
mod gpu;
mod specs;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{Json, Router, routing::get};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use serde_json::json;

use crate::auth::require_bearer;
use crate::config::Config;
use crate::specs::SpecsResponse;

#[derive(Parser, Debug)]
#[command(
    name = "sprout",
    version,
    about = "Hardware-info probe for LettuceAI runnability checks"
)]
struct Args {
    #[arg(
        long,
        help = "Path to the config file (created on first run if missing)"
    )]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = Args::parse();
    let config_path = args.config.unwrap_or_else(config::default_config_path);
    let (config, generated_key) = config::load_or_create(&config_path)?;

    if generated_key {
        print_key_banner(&config, &config_path);
    }

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let tls_paths = config.tls_paths(&config_path)?;
    let state = Arc::new(config);

    let protected =
        Router::new()
            .route("/specs", get(specs))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                require_bearer,
            ));

    let app = Router::new()
        .route("/health", get(health))
        .route("/ping", get(ping))
        .merge(protected);

    tracing::info!("config: {}", config_path.display());
    if !state.require_auth {
        tracing::warn!("auth disabled: /specs is served without a bearer token");
    }
    if let Some((cert_path, key_path)) = tls_paths {
        let tls = RustlsConfig::from_pem_file(&cert_path, &key_path).await?;
        tracing::info!("sprout listening on https://{addr}");
        axum_server::bind_rustls(addr, tls)
            .serve(app.into_make_service())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("sprout listening on http://{addr}");
        axum::serve(listener, app).await?;
    }
    Ok(())
}

fn print_key_banner(config: &Config, path: &Path) {
    println!();
    println!("  Sprout generated a new API key. Copy it now:");
    println!();
    println!("      {}", config.api_key);
    println!();
    println!("  Saved to {}", path.display());
    println!("  Authenticate requests with: Authorization: Bearer <key>");
    println!();
}

async fn health() -> &'static str {
    "ok"
}

async fn ping() -> Json<serde_json::Value> {
    Json(json!({ "service": "sprout", "version": env!("CARGO_PKG_VERSION") }))
}

async fn specs() -> Json<SpecsResponse> {
    Json(specs::collect())
}
