use stats_alloc::StatsAlloc;
use std::alloc::System;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

#[global_allocator]
static ALLOCATOR: StatsAlloc<System> = StatsAlloc::system();
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Router, routing::get};
use clap::Parser;
use serde::{Deserialize, Serialize};
use storage::common_types::{DataValue, Schema};
use storage::db::Database;
use tokio::net::TcpListener;

use crate::command_args::CommandArgs;

mod command_args;

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
    db: Arc<Database>,
}
impl DBState {
    pub fn new(db: Database) -> Self {
        Self { db: Arc::new(db) }
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ReturnValue {
    value: Vec<Vec<DataValue>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryQuery {
    sql: String,
}
pub async fn query(
    State(state): State<DBState>,
    Json(query): Json<QueryQuery>,
) -> Result<impl IntoResponse, String> {
    log::info!("Got query {}", query.sql);
    let instant = std::time::Instant::now();
    let query_req = parser::parse_query_request(query.sql.trim_matches('\"'))
        .map_err(|e| format!("Err:{:?}", e))?;
    log::info!(
        "Took {} s to parse query \"{}\"",
        instant.elapsed().as_secs_f32(),
        query.sql
    );
    let instant = std::time::Instant::now();
    let res = query_req.execute(&state.db);
    log::info!(
        "Took {} s to execute query \"{}\"",
        instant.elapsed().as_secs_f32(),
        query.sql
    );
    Ok(Json(res))
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Overview {
    overview: HashMap<String, Schema>,
}
pub async fn overview(State(state): State<DBState>) -> Result<impl IntoResponse, String> {
    log::info!("overview request");
    let mut overview = HashMap::new();
    for s in state.db.tables().iter() {
        overview.insert(s.0.clone(), s.1.schema().clone());
    }
    let res = Overview { overview };
    Ok(Json(res))
}

pub async fn health_handle() -> Json<serde_json::Value> {
    let time = chrono::Utc::now();
    let stats = ALLOCATOR.stats();

    let current_heap_size_kb = stats
        .bytes_allocated
        .saturating_sub(stats.bytes_deallocated)
        / 1024;
    serde_json::json!(
    {
        "status" : "healthy",
        "current_time" : time.to_rfc3339(),
        "allocated_memory": current_heap_size_kb,
    })
    .into()
}
pub async fn ping_handle() -> String {
    "pong".into()
}
