use net::objects::ParseErrorDTO;
use net::{objects::*, requests::*};
use simply_db::DatabaseContext;
use stats_alloc::StatsAlloc;
use std::alloc::System;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

#[global_allocator]
static ALLOCATOR: StatsAlloc<System> = StatsAlloc::system();
use axum::Json;
use axum::extract::State;
use axum::routing::post;
use axum::{Router, routing::get};
use clap::Parser;
use storage::db::Database;
use tokio::net::TcpListener;

use crate::command_args::CommandArgs;
use crate::metrics::ServerMetrics;

mod command_args;
mod metrics;

fn init_logger() {
    use env_logger::fmt::style::AnsiColor;
    use env_logger::{Builder, Env};
    use std::io::Write;
    let env = Env::default().filter_or("RUST_LOG", "trace");
    Builder::new()
        .format(|buf, record| {
            let time_style = AnsiColor::Cyan;
            let level_style = match record.level() {
                log::Level::Error => AnsiColor::BrightRed,
                log::Level::Warn => AnsiColor::Yellow,
                log::Level::Info => AnsiColor::Blue,
                log::Level::Debug => AnsiColor::Magenta,
                log::Level::Trace => AnsiColor::BrightGreen,
            };
            let default_level_style = AnsiColor::Black;
            writeln!(
                buf,
                "{}[{}]{}[{}]{}[{}] - {}",
                time_style.render_fg(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level_style.render_fg(),
                record.level(),
                default_level_style.render_fg(),
                record.target(),
                record.args()
            )
        })
        .parse_env(env)
        .init();
}
#[derive(Debug, Clone)]
pub struct DBState {
    db_context: Arc<DatabaseContext<ServerMetrics>>,
}
impl DBState {
    pub fn new(db: Database) -> Self {
        Self {
            db_context: Arc::new(DatabaseContext::new_with_metrics(db, ServerMetrics::new())),
        }
    }
}
async fn listen_ip() -> SocketAddr {
    dotenv::dotenv().ok();
    let env = std::env::var("LISTEN_IP");
    let args = CommandArgs::parse();
    let host = if let Some(host) = args.listen_ip() {
        host.clone()
    } else {
        if let Ok(host) = env {
            host
        } else {
            panic!(
                "Provide listen ip via either env file with LISTEN_IP or command argument --listen-ip"
            )
        }
    };
    log::info!("Listening on {}", host);
    tokio::net::lookup_host(&host)
        .await
        .expect("Expected valid address")
        .next()
        .expect("No such ip")
}
#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    init_logger();

    let ip = listen_ip().await;
    let db = Database::new();
    let router = Router::new()
        .route("/ping", get(ping_handle))
        .route("/health", get(health_handle))
        .route("/v1/query", post(query))
        .route("/v1/overview", get(overview))
        .with_state(DBState::new(db));
    let listener = TcpListener::bind(ip).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
#[axum::debug_handler]
pub async fn query(
    State(state): State<DBState>,
    Json(query): Json<SqlQueryRequest>,
) -> Result<Json<SqlQueryOutput>, ParseErrorDTO> {
    let res = state.db_context.execute(query.sql())?;
    Ok(Json(SqlQueryOutput::new(res)))
}
pub async fn overview(State(state): State<DBState>) -> Json<Overview> {
    log::info!("overview request");
    let mut overview_data = HashMap::new();
    for s in state.db_context.database().tables().iter() {
        overview_data.insert(s.0.clone(), s.1.schema().clone());
    }
    let res = Overview::new(overview_data);
    Json(res)
}
pub async fn health_handle() -> Json<Health> {
    let time = chrono::Utc::now();
    let stats = ALLOCATOR.stats();

    let current_heap_size_kb = stats
        .bytes_allocated
        .saturating_sub(stats.bytes_deallocated)
        / 1024;
    let health = Health::new(
        DatabaseState::Healthy,
        time.to_rfc3339(),
        MemoryMetrics::new(current_heap_size_kb),
    );
    health.into()
}
pub async fn ping_handle() -> String {
    "pong".into()
}
