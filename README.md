# SQL to AST API

一个使用 Rust 编写的高性能 HTTP 接口，用于将 SQL 语句转换为 AST（抽象语法树）并以 JSON 格式返回。

## 功能特性

- ✅ 将 SQL 语句解析为 AST
- ✅ 支持 8 种 SQL 方言（MySQL, PostgreSQL, SQLite, Hive, Snowflake, MSSQL, ANSI, Generic）
- ✅ 返回 JSON 格式的 AST
- ✅ 使用 Moka 实现高性能缓存（可配置容量和 TTL）
- ✅ 命令行参数配置缓存和监听端口
- ✅ OpenAPI 3.0 文档（Swagger UI）
- ✅ 健康检查接口
- ✅ 性能指标（请求耗时、缓存命中率）
- ✅ 支持禁用缓存的调试模式
- ✅ 精美的前端调试页面（离线可用）
- ✅ Docker 容器化部署
- ✅ 支持 CORS

## 快速开始

### 方式一：本地运行

```bash
# 克隆项目
git clone <repository-url>
cd sql-ast-api

# 运行服务
cargo run

# 访问前端页面
# http://127.0.0.1:3000
```

### 方式二：Docker 运行

```bash
# 使用 docker-compose（推荐）
docker-compose up -d

# 或使用 docker 命令
docker build -t sql-ast-api .
docker run -d -p 3000:3000 sql-ast-api
```

详细的 Docker 部署指南请查看 [DOCKER.md](DOCKER.md)

## 在线演示

启动服务后，打开浏览器访问：

- **前端调试页面**: http://127.0.0.1:3000
- **Swagger UI**: http://127.0.0.1:3000/swagger-ui
- **健康检查**: http://127.0.0.1:3000/health

### 前端页面特性

- 🎨 精美的双栏布局设计
- 📝 实时 SQL 编辑与解析
- 🌳 结构化的 AST 树展示
- 🎯 支持折叠/展开 JSON 节点
- ⚡ 实时性能指标显示
- 💾 缓存状态可视化
- 🎪 内置示例 SQL
- 🚫 支持禁用缓存调试
- 📱 响应式设计，移动端友好
- 🔌 完全离线可用（无外部依赖）

## 依赖

- **axum**: Web 框架
- **tokio**: 异步运行时
- **serde/serde_json**: JSON 序列化
- **sqlparser**: SQL 解析器
- **tower-http**: CORS 支持
- **moka**: 异步缓存库
- **clap**: 命令行参数解析
- **utoipa**: OpenAPI 文档生成
- **utoipa-swagger-ui**: Swagger UI 集成

## 安装与运行

### 编译

```bash
cargo build --release
```

### 运行（使用默认配置）

```bash
cargo run
```

或使用编译后的二进制文件：

```bash
./target/release/sql-ast-api
```

### 命令行参数

```bash
sql-ast-api [OPTIONS]

Options:
  --host <HOST>                          Server host [default: 127.0.0.1]
  -p, --port <PORT>                      Server port [default: 3000]
  --cache-max-capacity <CAPACITY>        Maximum cache entries [default: 10000]
  --cache-ttl <TTL>                      Cache TTL in seconds [default: 3600]
  -h, --help                             Print help
```

### 使用示例

```bash
# 使用默认配置
cargo run

# 自定义端口和缓存配置
cargo run -- --port 8080 --cache-max-capacity 5000 --cache-ttl 1800

# 监听所有网卡
cargo run -- --host 0.0.0.0 --port 8080
```

## API 文档

服务器启动后，访问以下 URL：

- **Swagger UI**: http://127.0.0.1:3000/swagger-ui
- **OpenAPI JSON**: http://127.0.0.1:3000/api-docs/openapi.json

## API 接口

### 1. 解析 SQL (POST /parse)

将 SQL 语句解析为 AST。

### 请求格式

```json
{
  "sql": "SELECT * FROM users WHERE id = 1",
  "dialect": "mysql",
  "no_cache": false
}
```

**参数说明：**
- `sql` (必需): 要解析的 SQL 语句
- `dialect` (可选): SQL 方言，默认为 "generic"
- `no_cache` (可选): 是否禁用缓存，默认为 false（启用缓存）

**支持的方言：**
- `generic` - 通用 SQL（默认）
- `mysql` - MySQL
- `postgresql` / `postgres` - PostgreSQL
- `sqlite` - SQLite
- `hive` - Apache Hive
- `snowflake` - Snowflake
- `mssql` / `sqlserver` - Microsoft SQL Server
- `ansi` - ANSI SQL

**成功响应 (200)：**

```json
{
  "ast": [
    {
      "Query": {
        "body": {
          "Select": {
            "projection": [...],
            "from": [...],
            "selection": {...}
          }
        }
      }
    }
  ],
  "cached": false,
  "elapsed_ms": 1.47
}
```

**响应字段：**
- `ast`: 解析后的 AST 结构
- `cached`: 是否从缓存中获取（true/false）
- `elapsed_ms`: 请求处理耗时（毫秒）

**错误响应 (400)：**

```json
{
  "error": "Failed to parse SQL: sql parser error: ...",
  "elapsed_ms": 0.18
}
```

### 2. 健康检查 (GET /health)

检查服务健康状态。

**响应 (200)：**

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

## 缓存机制

- **缓存键**: (SQL 语句, 方言) 组合
- **默认容量**: 10,000 条记录（可通过 `--cache-max-capacity` 配置）
- **默认过期时间**: 1 小时（可通过 `--cache-ttl` 配置）
- **缓存指示**: 响应中的 `cached` 字段表示是否命中缓存

相同的 SQL 语句和方言组合会被缓存，提高重复查询的性能。从缓存返回的请求通常在 0.1-0.5ms 内完成，而新解析的请求可能需要 1-5ms。

## 使用示例

### 使用 curl

```bash
# 解析 SQL（使用默认方言）
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1"}'

# 使用 MySQL 方言
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1", "dialect": "mysql"}'

# 禁用缓存（每次重新解析）
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1", "dialect": "mysql", "no_cache": true}'

# 健康检查
curl http://127.0.0.1:3000/health
```

### 使用 PowerShell

```powershell
# 解析 SQL
$body = @{
    sql = "SELECT * FROM users WHERE id = 1"
    dialect = "mysql"
} | ConvertTo-Json

$result = Invoke-RestMethod -Uri http://127.0.0.1:3000/parse `
    -Method Post `
    -ContentType "application/json" `
    -Body $body

Write-Host "Cached: $($result.cached), Time: $($result.elapsed_ms)ms"

# 禁用缓存
$body = @{
    sql = "SELECT * FROM users WHERE id = 1"
    dialect = "mysql"
    no_cache = $true
} | ConvertTo-Json

$result = Invoke-RestMethod -Uri http://127.0.0.1:3000/parse `
    -Method Post `
    -ContentType "application/json" `
    -Body $body

# 健康检查
Invoke-RestMethod -Uri http://127.0.0.1:3000/health
```

### 使用 Python

```python
import requests

# 解析 SQL
response = requests.post(
    "http://127.0.0.1:3000/parse",
    json={
        "sql": "SELECT * FROM users WHERE id = 1",
        "dialect": "postgresql"
    }
)

data = response.json()
print(f"Cached: {data['cached']}, Time: {data['elapsed_ms']}ms")
print(f"AST: {data['ast']}")

# 禁用缓存
response = requests.post(
    "http://127.0.0.1:3000/parse",
    json={
        "sql": "SELECT * FROM users WHERE id = 1",
        "dialect": "postgresql",
        "no_cache": True
    }
)

# 健康检查
health = requests.get("http://127.0.0.1:3000/health").json()
print(f"Status: {health['status']}, Version: {health['version']}")
```

## 性能特性

- **异步处理**: 基于 Tokio 异步运行时，支持高并发
- **高性能缓存**: Moka 提供线程安全的高性能并发访问
- **性能指标**: 
  - 缓存命中: ~0.1-0.5ms
  - 缓存未命中: ~1-5ms（取决于 SQL 复杂度）
  - 每秒可处理数千个请求
- **内存效率**: 可配置的缓存容量和 TTL

## 监控与日志

### 性能监控

每个响应都包含 `elapsed_ms` 字段，显示请求处理耗时：

```json
{
  "ast": {...},
  "cached": true,
  "elapsed_ms": 0.23
}
```

### 缓存监控

通过 `cached` 字段监控缓存命中率：
- `cached: false` - 新解析的 SQL，已存入缓存
- `cached: true` - 从缓存返回，性能最优

## 测试

项目包含一个 PowerShell 测试脚本 `test_api.ps1`，可以测试所有功能：

```powershell
# 启动服务器（在一个终端）
cargo run

# 运行测试（在另一个终端）
.\test_api.ps1
```

## Docker 支持

创建 `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sql-ast-api /usr/local/bin/
EXPOSE 3000
CMD ["sql-ast-api", "--host", "0.0.0.0"]
```

构建和运行：

```bash
# 使用 docker-compose（推荐）
docker-compose up -d

# 或使用 docker 命令
docker build -t sql-ast-api .
docker run -d -p 3000:3000 sql-ast-api

# 使用自定义配置
docker run -d -p 8080:8080 sql-ast-api \
  --host 0.0.0.0 \
  --port 8080 \
  --cache-max-capacity 50000 \
  --cache-ttl 7200
```

详细的 Docker 部署指南请查看 [DOCKER.md](DOCKER.md)

## 开发

### 运行开发服务器

```bash
cargo run
```

### 运行测试

```bash
cargo test
```

### 格式化代码

```bash
cargo fmt
```

### 检查代码

```bash
cargo clippy
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

