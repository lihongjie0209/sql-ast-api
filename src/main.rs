use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser as ClapParser;
use moka::future::Cache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlparser::dialect::{
    AnsiDialect, Dialect, GenericDialect, HiveDialect, MsSqlDialect, MySqlDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};
use sqlparser::parser::Parser;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

type CacheKey = (String, String);
type CacheValue = Result<serde_json::Value, String>;

// 全局 Dialect 缓存，避免重复创建
static DIALECTS: Lazy<HashMap<&'static str, Arc<dyn Dialect + Send + Sync>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "generic",
        Arc::new(GenericDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "mysql",
        Arc::new(MySqlDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "postgresql",
        Arc::new(PostgreSqlDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "postgres",
        Arc::new(PostgreSqlDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "sqlite",
        Arc::new(SQLiteDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "hive",
        Arc::new(HiveDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "snowflake",
        Arc::new(SnowflakeDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "mssql",
        Arc::new(MsSqlDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "sqlserver",
        Arc::new(MsSqlDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m.insert(
        "ansi",
        Arc::new(AnsiDialect {}) as Arc<dyn Dialect + Send + Sync>,
    );
    m
});

#[derive(ClapParser, Debug)]
#[command(name = "sql-ast-api")]
#[command(about = "SQL to AST API server", long_about = None)]
struct CliArgs {
    #[arg(long, default_value = "127.0.0.1", help = "Server host")]
    host: String,

    #[arg(short, long, default_value_t = 3000, help = "Server port")]
    port: u16,

    #[arg(
        long,
        default_value_t = 10000,
        help = "Maximum cache capacity (number of entries)"
    )]
    cache_max_capacity: u64,

    #[arg(
        long,
        default_value_t = 3600,
        help = "Cache TTL in seconds (time to live)"
    )]
    cache_ttl: u64,
}

#[derive(Clone)]
struct AppState {
    cache: Cache<CacheKey, CacheValue>,
}

#[derive(OpenApi)]
#[openapi(
    paths(parse_sql, health_check),
    components(schemas(SqlRequest, SqlResponse, ErrorResponse, HealthResponse))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct SqlRequest {
    #[schema(example = "SELECT * FROM users WHERE id = 1")]
    sql: String,

    #[serde(default = "default_dialect")]
    #[schema(example = "mysql", default = "generic")]
    dialect: String,

    #[serde(default)]
    #[schema(example = false, default = false)]
    no_cache: bool,
}

fn default_dialect() -> String {
    "generic".to_string()
}

#[derive(Serialize, ToSchema)]
struct SqlResponse {
    #[schema(value_type = Object)]
    ast: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = false)]
    cached: Option<bool>,

    #[schema(example = 1.234)]
    elapsed_ms: f64,
}

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    #[schema(example = "Failed to parse SQL: sql parser error: ...")]
    error: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 0.123)]
    elapsed_ms: Option<f64>,
}

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    #[schema(example = "ok")]
    status: String,

    #[schema(example = "0.1.0")]
    version: String,
}

enum ApiResponse {
    Success(SqlResponse),
    Error(ErrorResponse),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Success(response) => (StatusCode::OK, Json(response)).into_response(),
            ApiResponse::Error(error) => (StatusCode::BAD_REQUEST, Json(error)).into_response(),
        }
    }
}

fn get_dialect(dialect_name: &str) -> Result<Arc<dyn Dialect + Send + Sync>, String> {
    DIALECTS
        .get(dialect_name.to_lowercase().as_str())
        .cloned()
        .ok_or_else(|| {
            format!(
                "Unsupported dialect: {dialect_name}. Supported dialects: generic, mysql, postgresql, sqlite, hive, snowflake, mssql, ansi"
            )
        })
}

// SQL 规范化，提高缓存命中率
fn normalize_sql(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

async fn parse_sql_impl(sql: &str, dialect_name: &str) -> CacheValue {
    let dialect = match get_dialect(dialect_name) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };

    match Parser::parse_sql(&*dialect, sql) {
        Ok(ast) => match serde_json::to_value(&ast) {
            Ok(json_ast) => Ok(json_ast),
            Err(e) => Err(format!("Failed to serialize AST: {e}")),
        },
        Err(e) => Err(format!("Failed to parse SQL: {e}")),
    }
}

#[utoipa::path(
    post,
    path = "/parse",
    request_body = SqlRequest,
    responses(
        (status = 200, description = "SQL parsed successfully", body = SqlResponse),
        (status = 400, description = "Invalid SQL or unsupported dialect", body = ErrorResponse)
    ),
    tag = "SQL Parser"
)]
async fn parse_sql(State(state): State<AppState>, Json(payload): Json<SqlRequest>) -> ApiResponse {
    let start = Instant::now();

    // SQL 规范化，提高缓存命中率
    let normalized_sql = if payload.no_cache {
        payload.sql.clone()
    } else {
        normalize_sql(&payload.sql)
    };

    let cache_key = (normalized_sql.clone(), payload.dialect.clone());

    // 如果未禁用缓存，先尝试从缓存获取
    if !payload.no_cache {
        if let Some(cached_result) = state.cache.get(&cache_key).await {
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            match cached_result {
                Ok(ast) => {
                    return ApiResponse::Success(SqlResponse {
                        ast,
                        cached: Some(true),
                        elapsed_ms: elapsed,
                    });
                }
                Err(e) => {
                    return ApiResponse::Error(ErrorResponse {
                        error: e,
                        elapsed_ms: Some(elapsed),
                    });
                }
            }
        }
    }

    let result = parse_sql_impl(&normalized_sql, &payload.dialect).await;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    // 只有在未禁用缓存时才存入缓存
    if !payload.no_cache {
        state.cache.insert(cache_key, result.clone()).await;
    }

    match result {
        Ok(ast) => ApiResponse::Success(SqlResponse {
            ast,
            cached: Some(false),
            elapsed_ms: elapsed,
        }),
        Err(e) => ApiResponse::Error(ErrorResponse {
            error: e,
            elapsed_ms: Some(elapsed),
        }),
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "Health"
)]
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let cache = Cache::builder()
        .max_capacity(args.cache_max_capacity)
        .time_to_live(Duration::from_secs(args.cache_ttl))
        .build();

    let state = AppState { cache };

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/parse", post(parse_sql))
        .route("/health", get(health_check))
        .route("/", get(serve_index))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from((
        args.host.parse::<std::net::IpAddr>().unwrap_or_else(|_| {
            eprintln!("Invalid host address, using 127.0.0.1");
            "127.0.0.1".parse().unwrap()
        }),
        args.port,
    ));

    println!("🚀 SQL to AST API Server v{}", env!("CARGO_PKG_VERSION"));
    println!("📡 Server running on http://{addr}");
    println!("📚 OpenAPI docs: http://{addr}/swagger-ui");
    println!("❤️  Health check: http://{addr}/health");
    println!();
    println!("⚙️  Configuration:");
    println!("   - Cache capacity: {}", args.cache_max_capacity);
    println!("   - Cache TTL: {}s", args.cache_ttl);
    println!();
    println!("📖 API Endpoints:");
    println!("   POST /parse - Parse SQL to AST");
    println!("   GET  /health - Health check");
    println!();
    println!(
        "🎯 Supported dialects: generic, mysql, postgresql, sqlite, hive, snowflake, mssql, ansi"
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
