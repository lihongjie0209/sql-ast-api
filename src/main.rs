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
use sqlparser::ast::{Expr, Statement, Value, VisitMut, VisitorMut};
use sqlparser::dialect::{
    AnsiDialect, Dialect, GenericDialect, HiveDialect, MsSqlDialect, MySqlDialect,
    PostgreSqlDialect, SQLiteDialect, SnowflakeDialect,
};
use sqlparser::parser::Parser;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tonic::{transport::Server, Request, Response as TonicResponse, Status};
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

// Include generated gRPC code
pub mod sql_parser {
    tonic::include_proto!("sql_parser");
}

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

    #[arg(short, long, default_value_t = 3000, help = "HTTP server port")]
    port: u16,

    #[arg(long, default_value_t = 50051, help = "gRPC server port")]
    grpc_port: u16,

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
    paths(parse_sql, fingerprint_sql, health_check),
    components(schemas(SqlRequest, SqlResponse, ErrorResponse, HealthResponse, FingerprintRequest, FingerprintResponse))
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

#[derive(Deserialize, ToSchema)]
struct FingerprintRequest {
    #[schema(example = "SELECT * FROM users WHERE id = 123 AND name = 'John' AND age IN (25, 30, 35, 40)")]
    sql: String,

    #[serde(default = "default_dialect")]
    #[schema(example = "mysql", default = "generic")]
    dialect: String,

    #[serde(default)]
    #[schema(example = 3, default = 0)]
    max_in_values: usize,
}

#[derive(Serialize, ToSchema)]
struct FingerprintResponse {
    #[schema(example = "SELECT * FROM users WHERE id = ? AND name = ? AND age IN (?, ?, ?)")]
    fingerprint: String,

    #[schema(example = 1.234)]
    elapsed_ms: f64,
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

enum FingerprintApiResponse {
    Success(FingerprintResponse),
    Error(ErrorResponse),
}

impl IntoResponse for FingerprintApiResponse {
    fn into_response(self) -> Response {
        match self {
            FingerprintApiResponse::Success(response) => (StatusCode::OK, Json(response)).into_response(),
            FingerprintApiResponse::Error(error) => (StatusCode::BAD_REQUEST, Json(error)).into_response(),
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

// SQL指纹生成器：将字面量替换为?
struct FingerprintVisitor {
    max_in_values: usize,
}

impl VisitorMut for FingerprintVisitor {
    type Break = ();

    fn pre_visit_expr(&mut self, expr: &mut Expr) -> std::ops::ControlFlow<Self::Break> {
        match expr {
            // 将所有字面量值替换为占位符
            Expr::Value(Value::Number(_, _))
            | Expr::Value(Value::SingleQuotedString(_))
            | Expr::Value(Value::DoubleQuotedString(_))
            | Expr::Value(Value::NationalStringLiteral(_))
            | Expr::Value(Value::HexStringLiteral(_))
            | Expr::Value(Value::Boolean(_)) => {
                *expr = Expr::Value(Value::Placeholder("?".to_string()));
            }
            // 处理IN表达式，限制列表长度
            Expr::InList { expr: inner_expr, list, negated: _ } => {
                // 先递归处理内部表达式
                let _ = self.pre_visit_expr(inner_expr);
                
                // 限制IN列表的长度
                if self.max_in_values > 0 && list.len() > self.max_in_values {
                    list.truncate(self.max_in_values);
                }
                
                // 将列表中的每个表达式转换为占位符
                for item in list.iter_mut() {
                    let _ = self.pre_visit_expr(item);
                }
                
                return std::ops::ControlFlow::Continue(());
            }
            _ => {}
        }
        std::ops::ControlFlow::Continue(())
    }
}

fn generate_sql_fingerprint(mut statements: Vec<Statement>, max_in_values: usize) -> String {
    let mut visitor = FingerprintVisitor { max_in_values };
    
    for stmt in statements.iter_mut() {
        let _ = stmt.visit(&mut visitor);
    }
    
    statements
        .iter()
        .map(|stmt| stmt.to_string())
        .collect::<Vec<_>>()
        .join("; ")
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
    post,
    path = "/fingerprint",
    request_body = FingerprintRequest,
    responses(
        (status = 200, description = "SQL fingerprint generated successfully", body = FingerprintResponse),
        (status = 400, description = "Invalid SQL or unsupported dialect", body = ErrorResponse)
    ),
    tag = "SQL Fingerprint"
)]
async fn fingerprint_sql(Json(payload): Json<FingerprintRequest>) -> FingerprintApiResponse {
    let start = Instant::now();
    
    let dialect = match get_dialect(&payload.dialect) {
        Ok(d) => d,
        Err(e) => {
            return FingerprintApiResponse::Error(ErrorResponse {
                error: e,
                elapsed_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
            });
        }
    };
    
    match Parser::parse_sql(&*dialect, &payload.sql) {
        Ok(statements) => {
            let fingerprint = generate_sql_fingerprint(statements, payload.max_in_values);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            
            FingerprintApiResponse::Success(FingerprintResponse {
                fingerprint,
                elapsed_ms: elapsed,
            })
        }
        Err(e) => FingerprintApiResponse::Error(ErrorResponse {
            error: format!("Failed to parse SQL: {e}"),
            elapsed_ms: Some(start.elapsed().as_secs_f64() * 1000.0),
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

// gRPC Service Implementation
pub struct SqlParserGrpcService {
    cache: Cache<CacheKey, CacheValue>,
}

#[tonic::async_trait]
impl sql_parser::sql_parser_service_server::SqlParserService for SqlParserGrpcService {
    async fn parse_sql(
        &self,
        request: Request<sql_parser::ParseSqlRequest>,
    ) -> Result<TonicResponse<sql_parser::ParseSqlResponse>, Status> {
        let req = request.into_inner();
        let start = Instant::now();

        let normalized_sql = if req.no_cache {
            req.sql.clone()
        } else {
            normalize_sql(&req.sql)
        };

        let cache_key = (normalized_sql.clone(), req.dialect.clone());

        // Check cache if not disabled
        if !req.no_cache {
            if let Some(cached_result) = self.cache.get(&cache_key).await {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                return match cached_result {
                    Ok(ast) => Ok(TonicResponse::new(sql_parser::ParseSqlResponse {
                        result: Some(sql_parser::parse_sql_response::Result::Success(
                            sql_parser::ParseSqlSuccess {
                                ast_json: ast.to_string(),
                                cached: true,
                                elapsed_ms: elapsed,
                            },
                        )),
                    })),
                    Err(e) => Ok(TonicResponse::new(sql_parser::ParseSqlResponse {
                        result: Some(sql_parser::parse_sql_response::Result::Error(
                            sql_parser::ParseSqlError {
                                error_message: e,
                                elapsed_ms: elapsed,
                            },
                        )),
                    })),
                };
            }
        }

        let result = parse_sql_impl(&normalized_sql, &req.dialect).await;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        if !req.no_cache {
            self.cache.insert(cache_key, result.clone()).await;
        }

        match result {
            Ok(ast) => Ok(TonicResponse::new(sql_parser::ParseSqlResponse {
                result: Some(sql_parser::parse_sql_response::Result::Success(
                    sql_parser::ParseSqlSuccess {
                        ast_json: ast.to_string(),
                        cached: false,
                        elapsed_ms: elapsed,
                    },
                )),
            })),
            Err(e) => Ok(TonicResponse::new(sql_parser::ParseSqlResponse {
                result: Some(sql_parser::parse_sql_response::Result::Error(
                    sql_parser::ParseSqlError {
                        error_message: e,
                        elapsed_ms: elapsed,
                    },
                )),
            })),
        }
    }

    async fn generate_fingerprint(
        &self,
        request: Request<sql_parser::FingerprintRequest>,
    ) -> Result<TonicResponse<sql_parser::FingerprintResponse>, Status> {
        let req = request.into_inner();
        let start = Instant::now();

        let dialect = match get_dialect(&req.dialect) {
            Ok(d) => d,
            Err(e) => {
                return Ok(TonicResponse::new(sql_parser::FingerprintResponse {
                    result: Some(sql_parser::fingerprint_response::Result::Error(
                        sql_parser::FingerprintError {
                            error_message: e,
                            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                        },
                    )),
                }));
            }
        };

        match Parser::parse_sql(&*dialect, &req.sql) {
            Ok(statements) => {
                let fingerprint = generate_sql_fingerprint(statements, req.max_in_values as usize);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;

                Ok(TonicResponse::new(sql_parser::FingerprintResponse {
                    result: Some(sql_parser::fingerprint_response::Result::Success(
                        sql_parser::FingerprintSuccess {
                            fingerprint,
                            elapsed_ms: elapsed,
                        },
                    )),
                }))
            }
            Err(e) => Ok(TonicResponse::new(sql_parser::FingerprintResponse {
                result: Some(sql_parser::fingerprint_response::Result::Error(
                    sql_parser::FingerprintError {
                        error_message: format!("Failed to parse SQL: {e}"),
                        elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                    },
                )),
            })),
        }
    }

    async fn health_check(
        &self,
        _request: Request<sql_parser::HealthCheckRequest>,
    ) -> Result<TonicResponse<sql_parser::HealthCheckResponse>, Status> {
        Ok(TonicResponse::new(sql_parser::HealthCheckResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let cache = Cache::builder()
        .max_capacity(args.cache_max_capacity)
        .time_to_live(Duration::from_secs(args.cache_ttl))
        .build();

    let state = AppState {
        cache: cache.clone(),
    };

    // HTTP Server setup
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/parse", post(parse_sql))
        .route("/fingerprint", post(fingerprint_sql))
        .route("/health", get(health_check))
        .route("/", get(serve_index))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let http_addr = SocketAddr::from((
        args.host.parse::<std::net::IpAddr>().unwrap_or_else(|_| {
            eprintln!("Invalid host address, using 127.0.0.1");
            "127.0.0.1".parse().unwrap()
        }),
        args.port,
    ));

    // gRPC Server setup
    let grpc_addr = SocketAddr::from((
        args.host.parse::<std::net::IpAddr>().unwrap_or_else(|_| {
            "127.0.0.1".parse().unwrap()
        }),
        args.grpc_port,
    ));

    let grpc_service = SqlParserGrpcService { cache };

    println!("🚀 SQL to AST API Server v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("📡 HTTP Server running on http://{}", http_addr);
    println!("   📚 OpenAPI docs: http://{}/swagger-ui", http_addr);
    println!("   ❤️  Health check: http://{}/health", http_addr);
    println!();
    println!("🔌 gRPC Server running on http://{}", grpc_addr);
    println!();
    println!("⚙️  Configuration:");
    println!("   - Cache capacity: {}", args.cache_max_capacity);
    println!("   - Cache TTL: {}s", args.cache_ttl);
    println!();
    println!("📖 HTTP API Endpoints:");
    println!("   POST /parse - Parse SQL to AST");
    println!("   POST /fingerprint - Generate SQL fingerprint");
    println!("   GET  /health - Health check");
    println!();
    println!("📖 gRPC Services:");
    println!("   ParseSql - Parse SQL to AST");
    println!("   GenerateFingerprint - Generate SQL fingerprint");
    println!("   HealthCheck - Health check");
    println!();
    println!(
        "🎯 Supported dialects: generic, mysql, postgresql, sqlite, hive, snowflake, mssql, ansi"
    );

    // Start both servers concurrently
    let http_server = async {
        let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    };

    let grpc_server = async {
        Server::builder()
            .add_service(sql_parser::sql_parser_service_server::SqlParserServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
            .unwrap();
    };

    tokio::select! {
        _ = http_server => {},
        _ = grpc_server => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::parser::Parser;

    #[test]
    fn test_fingerprint_basic_select() {
        let dialect = MySqlDialect {};
        let sql = "SELECT * FROM users WHERE id = 123 AND name = 'John'";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "SELECT * FROM users WHERE id = ? AND name = ?");
    }

    #[test]
    fn test_fingerprint_with_in_clause() {
        let dialect = MySqlDialect {};
        let sql = "SELECT * FROM users WHERE age IN (25, 30, 35, 40, 45)";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        // Test with no limit
        let fingerprint = generate_sql_fingerprint(statements.clone(), 0);
        assert_eq!(fingerprint, "SELECT * FROM users WHERE age IN (?, ?, ?, ?, ?)");
        
        // Test with limit of 3
        let fingerprint = generate_sql_fingerprint(statements, 3);
        assert_eq!(fingerprint, "SELECT * FROM users WHERE age IN (?, ?, ?)");
    }

    #[test]
    fn test_fingerprint_update_statement() {
        let dialect = MySqlDialect {};
        let sql = "UPDATE products SET price = 99.99, quantity = 100 WHERE id = 1";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "UPDATE products SET price = ?, quantity = ? WHERE id = ?");
    }

    #[test]
    fn test_fingerprint_delete_statement() {
        let dialect = PostgreSqlDialect {};
        let sql = "DELETE FROM logs WHERE created_at < '2024-01-01' AND status = 'archived'";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "DELETE FROM logs WHERE created_at < ? AND status = ?");
    }

    #[test]
    fn test_fingerprint_insert_statement() {
        let dialect = MySqlDialect {};
        let sql = "INSERT INTO products (name, price, quantity) VALUES ('Laptop', 999.99, 50)";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "INSERT INTO products (name, price, quantity) VALUES (?, ?, ?)");
    }

    #[test]
    fn test_fingerprint_complex_query() {
        let dialect = MySqlDialect {};
        let sql = "SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id WHERE u.age > 18 AND o.status = 'completed'";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "SELECT u.name, o.total FROM users AS u JOIN orders AS o ON u.id = o.user_id WHERE u.age > ? AND o.status = ?");
    }

    #[test]
    fn test_fingerprint_between_clause() {
        let dialect = MySqlDialect {};
        let sql = "SELECT * FROM products WHERE price BETWEEN 100 AND 500";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "SELECT * FROM products WHERE price BETWEEN ? AND ?");
    }

    #[test]
    fn test_fingerprint_with_multiple_in_clauses() {
        let dialect = MySqlDialect {};
        let sql = "SELECT * FROM products WHERE category_id IN (1, 2, 3, 4, 5) AND status_id IN (10, 20, 30)";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        // Test with limit of 2 for each IN clause
        let fingerprint = generate_sql_fingerprint(statements, 2);
        assert_eq!(fingerprint, "SELECT * FROM products WHERE category_id IN (?, ?) AND status_id IN (?, ?)");
    }

    #[test]
    fn test_fingerprint_preserves_null() {
        let dialect = MySqlDialect {};
        let sql = "SELECT * FROM users WHERE email IS NULL";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "SELECT * FROM users WHERE email IS NULL");
    }

    #[test]
    fn test_fingerprint_case_expression() {
        let dialect = MySqlDialect {};
        let sql = "SELECT CASE WHEN age > 18 THEN 'adult' ELSE 'minor' END FROM users";
        let statements = Parser::parse_sql(&dialect, sql).unwrap();
        
        let fingerprint = generate_sql_fingerprint(statements, 0);
        assert_eq!(fingerprint, "SELECT CASE WHEN age > ? THEN ? ELSE ? END FROM users");
    }

    #[test]
    fn test_normalize_sql() {
        let sql = "SELECT   *   FROM    users   WHERE   id =  1";
        let normalized = normalize_sql(sql);
        assert_eq!(normalized, "SELECT * FROM users WHERE id = 1");
    }

    #[test]
    fn test_get_dialect() {
        assert!(get_dialect("mysql").is_ok());
        assert!(get_dialect("postgresql").is_ok());
        assert!(get_dialect("MYSQL").is_ok()); // case insensitive
        assert!(get_dialect("invalid_dialect").is_err());
    }
}
