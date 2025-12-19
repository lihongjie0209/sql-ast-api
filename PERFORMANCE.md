# SQL Parser 性能优化指南

## 当前性能基准

当前实现的性能：
- 缓存命中: ~0.05-0.2ms
- 缓存未命中: ~0.2-2ms
- 复杂查询: ~0.5-5ms

## SqlParser 性能优化策略

### 1. 编译优化

#### 1.1 启用 Release 模式优化

在 `Cargo.toml` 中配置：

```toml
[profile.release]
opt-level = 3           # 最高优化级别
lto = "fat"             # 链接时优化（Link Time Optimization）
codegen-units = 1       # 减少代码生成单元，增加优化
panic = 'abort'         # 减小二进制大小
strip = true            # 剥离符号表
```

**性能提升**: 10-30%

#### 1.2 启用 CPU 特性优化

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1

# 针对本地 CPU 优化（不可移植）
[build]
rustflags = ["-C", "target-cpu=native"]
```

**性能提升**: 5-15%

### 2. Parser 复用策略

#### 2.1 避免重复创建 Dialect

**当前实现（低效）:**
```rust
async fn parse_sql_impl(sql: &str, dialect_name: &str) -> CacheValue {
    let dialect = match get_dialect(dialect_name) {  // 每次都创建
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    Parser::parse_sql(&*dialect, sql)
}
```

**优化方案 1: Dialect 缓存**

```rust
use once_cell::sync::Lazy;
use std::collections::HashMap;

static DIALECTS: Lazy<HashMap<&'static str, Box<dyn Dialect + Send + Sync>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("generic", Box::new(GenericDialect {}) as Box<dyn Dialect + Send + Sync>);
    m.insert("mysql", Box::new(MySqlDialect {}) as Box<dyn Dialect + Send + Sync>);
    // ... 其他方言
    m
});

async fn parse_sql_impl(sql: &str, dialect_name: &str) -> CacheValue {
    let dialect = DIALECTS.get(dialect_name)
        .ok_or_else(|| format!("Unsupported dialect: {}", dialect_name))?;
    
    Parser::parse_sql(&**dialect, sql)
}
```

**性能提升**: 5-10%（避免重复分配）

**优化方案 2: 使用 Arc 共享**

```rust
use std::sync::Arc;

#[derive(Clone)]
struct DialectRef(Arc<dyn Dialect + Send + Sync>);

static DIALECTS: Lazy<HashMap<&'static str, DialectRef>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("generic", DialectRef(Arc::new(GenericDialect {})));
    m.insert("mysql", DialectRef(Arc::new(MySqlDialect {})));
    // ...
    m
});
```

### 3. 缓存优化

#### 3.1 使用 AST 而非 JSON 缓存

**当前实现:**
```rust
type CacheValue = Result<serde_json::Value, String>;  // 缓存序列化后的 JSON
```

**优化方案:**
```rust
use sqlparser::ast::Statement;

type CacheValue = Result<Vec<Statement>, String>;  // 缓存原始 AST

// 只在返回时序列化
async fn parse_sql(...) -> ApiResponse {
    // 获取 AST
    let ast = get_from_cache_or_parse(...).await?;
    
    // 延迟序列化
    let json_ast = serde_json::to_value(&ast)?;
    
    ApiResponse::Success(SqlResponse { ast: json_ast, ... })
}
```

**优势**:
- 减少缓存内存占用
- 避免重复序列化
- 更快的缓存存储

**性能提升**: 10-20%

#### 3.2 实现预热缓存

```rust
async fn warmup_cache(state: &AppState) {
    let common_queries = vec![
        ("SELECT * FROM users", "mysql"),
        ("SELECT COUNT(*) FROM orders", "postgresql"),
        // ... 常见查询
    ];
    
    for (sql, dialect) in common_queries {
        let _ = parse_sql_impl(sql, dialect).await;
    }
}

#[tokio::main]
async fn main() {
    let state = AppState { cache };
    
    // 预热缓存
    warmup_cache(&state).await;
    
    // 启动服务
    // ...
}
```

### 4. 并行处理优化

#### 4.1 批量解析接口

```rust
#[derive(Deserialize)]
struct BatchSqlRequest {
    queries: Vec<SqlQuery>,
}

#[derive(Deserialize)]
struct SqlQuery {
    sql: String,
    dialect: String,
}

async fn parse_sql_batch(
    State(state): State<AppState>,
    Json(payload): Json<BatchSqlRequest>,
) -> Json<Vec<SqlResponse>> {
    use futures::stream::{self, StreamExt};
    
    let results = stream::iter(payload.queries)
        .map(|query| async {
            parse_sql_impl(&query.sql, &query.dialect).await
        })
        .buffer_unordered(10)  // 并行处理 10 个
        .collect::<Vec<_>>()
        .await;
    
    Json(results.into_iter().map(|r| /* 转换 */).collect())
}
```

**性能提升**: 批量处理可提升 3-10 倍吞吐量

### 5. 内存优化

#### 5.1 使用 String Interning

对于重复出现的字符串（如表名、列名），使用字符串驻留：

```rust
use string_cache::DefaultAtom as Atom;

// 在处理 AST 时使用
fn intern_strings(ast: &mut Statement) {
    // 将频繁出现的字符串转换为 Atom
    // 减少内存占用和比较开销
}
```

#### 5.2 使用 Compact 数据结构

```rust
use compact_str::CompactString;

// 对于短字符串（<24字节）使用 CompactString
// 避免堆分配
```

### 6. SQL 预处理

#### 6.1 SQL 规范化

```rust
fn normalize_sql(sql: &str) -> String {
    sql.trim()
        .replace('\n', " ")
        .replace('\t', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()  // 或 to_lowercase()
}

// 规范化后的 SQL 有更高的缓存命中率
let normalized = normalize_sql(&payload.sql);
let cache_key = (normalized, payload.dialect.clone());
```

**优势**: 提高缓存命中率 20-40%

#### 6.2 参数化查询支持

```rust
// 将具体值替换为占位符
fn parameterize_sql(sql: &str) -> (String, Vec<String>) {
    // "SELECT * FROM users WHERE id = 1"
    // -> ("SELECT * FROM users WHERE id = ?", vec!["1"])
}

// 参数化的 SQL 有更高的缓存命中率
```

### 7. 智能缓存策略

#### 7.1 LFU（最少使用频率）缓存

```rust
use moka::future::CacheBuilder;

let cache = CacheBuilder::new(10_000)
    .time_to_live(Duration::from_secs(3600))
    .eviction_listener(|key, value, cause| {
        // 记录被驱逐的项
        println!("Evicted: {:?}", key);
    })
    .build();
```

#### 7.2 分层缓存

```rust
struct TieredCache {
    hot: Cache<CacheKey, CacheValue>,    // 小而快（内存）
    warm: Cache<CacheKey, CacheValue>,   // 中等大小
    // cold: Redis/Disk cache             // 大而慢（可选）
}

// 根据访问频率自动调整
```

### 8. 异步优化

#### 8.1 使用 Rayon 进行 CPU 密集型任务

```rust
use rayon::prelude::*;

async fn parse_sql_cpu_intensive(sql: &str, dialect: &str) -> Result<...> {
    let sql = sql.to_string();
    let dialect = dialect.to_string();
    
    // 在线程池中执行 CPU 密集型解析
    tokio::task::spawn_blocking(move || {
        let dialect = get_dialect(&dialect)?;
        Parser::parse_sql(&*dialect, &sql)
    })
    .await?
}
```

**适用场景**: 复杂 SQL、大批量处理

**性能提升**: 20-50%（对于复杂查询）

### 9. 监控和分析

#### 9.1 添加性能追踪

```rust
use tracing::{instrument, info};

#[instrument(skip(state))]
async fn parse_sql(
    State(state): State<AppState>,
    Json(payload): Json<SqlRequest>,
) -> ApiResponse {
    let start = Instant::now();
    
    // ... 解析逻辑
    
    let elapsed = start.elapsed();
    info!("Parse completed in {:?}", elapsed);
    
    // 记录慢查询
    if elapsed > Duration::from_millis(10) {
        warn!("Slow query: {} ms", elapsed.as_millis());
    }
}
```

#### 9.2 添加指标收集

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref PARSE_DURATION: Histogram = register_histogram!(
        "sql_parse_duration_seconds",
        "SQL parsing duration"
    ).unwrap();
    
    static ref CACHE_HITS: Counter = register_counter!(
        "sql_cache_hits_total",
        "Cache hit count"
    ).unwrap();
}

// 在解析时记录
PARSE_DURATION.observe(elapsed.as_secs_f64());
if cached { CACHE_HITS.inc(); }
```

### 10. 实施建议

#### 优先级排序

**高优先级（立即实施）:**
1. ✅ Release 模式编译优化
2. ✅ Dialect 对象复用
3. ✅ SQL 规范化提高缓存命中率
4. ✅ 监控和慢查询日志

**中优先级（逐步实施）:**
5. AST 缓存替代 JSON 缓存
6. 预热常见查询
7. CPU 密集型任务使用 spawn_blocking

**低优先级（按需实施）:**
8. 批量处理接口
9. 分层缓存
10. 参数化查询

### 11. 性能测试对比

创建基准测试文件 `benches/parser_bench.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlparser::{dialect::GenericDialect, parser::Parser};

fn parse_simple(c: &mut Criterion) {
    let sql = "SELECT * FROM users WHERE id = 1";
    let dialect = GenericDialect {};
    
    c.bench_function("parse_simple", |b| {
        b.iter(|| Parser::parse_sql(&dialect, black_box(sql)))
    });
}

fn parse_complex(c: &mut Criterion) {
    let sql = "SELECT u.id, COUNT(o.id) FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.id";
    let dialect = GenericDialect {};
    
    c.bench_function("parse_complex", |b| {
        b.iter(|| Parser::parse_sql(&dialect, black_box(sql)))
    });
}

criterion_group!(benches, parse_simple, parse_complex);
criterion_main!(benches);
```

在 `Cargo.toml` 中添加：

```toml
[[bench]]
name = "parser_bench"
harness = false

[dev-dependencies]
criterion = "0.5"
```

运行基准测试：

```bash
cargo bench
```

### 12. 预期性能提升

应用上述优化后的预期性能：

| 优化项 | 当前 | 优化后 | 提升 |
|--------|------|--------|------|
| 简单查询（缓存未命中） | 0.2-2ms | 0.1-1ms | 50% |
| 复杂查询（缓存未命中） | 0.5-5ms | 0.3-3ms | 40% |
| 缓存命中 | 0.05-0.2ms | 0.02-0.1ms | 50% |
| 内存占用 | 基准 | -30% | 节省30% |
| 缓存命中率 | 60% | 80% | +33% |

### 总结

实施上述优化的建议顺序：

1. **立即实施** (1-2小时):
   - Release 编译优化配置
   - Dialect 对象复用
   - 基础监控

2. **短期实施** (1-2天):
   - SQL 规范化
   - AST 缓存
   - 慢查询日志

3. **中期实施** (1周):
   - 预热缓存
   - CPU 密集型优化
   - 完整的指标收集

4. **长期实施** (按需):
   - 批量处理
   - 分层缓存
   - 高级优化策略

预期综合性能提升：**30-100%**
