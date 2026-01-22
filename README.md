# SQL Parser & Fingerprint API

一个使用 Rust 编写的高性能 API 服务，支持 SQL 解析为 AST 和 SQL 指纹生成，同时提供 HTTP REST API 和 gRPC 接口。

## 功能特性

- ✅ **SQL 解析**: 将 SQL 语句解析为 AST（抽象语法树）
- ✅ **SQL 指纹**: 生成标准化的 SQL 模板，支持限制 IN 子句值数量
- ✅ **双协议支持**: HTTP REST API 和 gRPC 服务
- ✅ **8 种 SQL 方言**: MySQL, PostgreSQL, SQLite, Hive, Snowflake, MSSQL, ANSI, Generic
- ✅ **高性能缓存**: 使用 Moka 实现并发安全的缓存（可配置容量和 TTL）
- ✅ **OpenAPI 文档**: Swagger UI 支持
- ✅ **精美 Web 界面**: 支持 AST 解析和指纹生成
- ✅ **单元测试**: 12 个测试用例覆盖核心功能
- ✅ **Docker 支持**: 容器化部署
- ✅ **CORS 支持**: 跨域资源共享

## 快速开始

### 方式一：本地运行

```bash
# 克隆项目
git clone https://github.com/lihongjie0209/sql-ast-api.git
cd sql-ast-api

# 运行服务（同时启动 HTTP 和 gRPC）
cargo run

# 访问服务
# HTTP: http://127.0.0.1:3000
# gRPC: http://127.0.0.1:50051
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
- **gRPC 服务**: http://127.0.0.1:50051

### 前端页面特性

- 🎨 精美的双栏布局设计
- 📝 实时 SQL 编辑与解析
- 🌳 结构化的 AST 树展示
- 🔍 SQL 指纹生成功能
- 🎯 支持折叠/展开 JSON 节点
- ⚡ 实时性能指标显示
- 💾 缓存状态可视化
- 🎪 内置示例 SQL
- 🚫 支持禁用缓存调试
- 🔢 可配置 IN 子句最大值数量
- 📱 响应式设计，移动端友好
- 🔌 完全离线可用（无外部依赖）
### HTTP API
- **axum**: Web 框架
- **tokio**: 异步运行时
- **serde/serde_json**: JSON 序列化
- **sqlparser**: SQL 解析器
- **tower-http**: CORS 支持
- **moka**: 异步缓存库
- **clap**: 命令行参数解析
- **utoipa**: OpenAPI 文档生成
- **utoipa-swagger-ui**: Swagger UI 集成

### gRPC
- **tonic**: gRPC 框架
- **prost**: Protocol Buffers 实现
- **tonic-build**: proto 文件编译
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

```bashHTTP server port [default: 3000]
  --grpc-port <GRPC_PORT>                gRPC server port [default: 50051]
  --cache-max-capacity <CAPACITY>        Maximum cache entries [default: 10000]
  --cache-ttl <TTL>                      Cache TTL in seconds [default: 3600]
  -h, --help                             Print help
```

### 使用示例

```bash
# 使用默认配置（HTTP:3000, gRPC:50051）
cargo run

# 自定义端口和缓存配置
cargo run -- --port 8080 --grpc-port 50052 --cache-max-capacity 5000 --cache-ttl 1800

# 监听所有网卡
cargo run -- --host 0.0.0.0 --port 8080 --grpc-port 50051
### 使用示例

```bash
# 使用默认配置
### HTTP REST API

服务器启动后，访问以下 URL：

- **Swagger UI**: http://127.0.0.1:3000/swagger-ui
- **OpenAPI JSON**: http://127.0.0.1:3000/api-docs/openapi.json

### gRPC API

gRPC 服务定义在 `proto/sql_parser.proto`，包含以下 RPC 方法：
- `ParseSql`: 解析 SQL 为 AST
- `GenerateFingerprint`: 生成 SQL 指纹
- `HealthCheck`: 健康检查

## HTTP API 接口

### 1. 解析 SQL (POST /parse)

将 SQL 语句解析为 AST。

**请求格式:**

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

### 2. 生成 SQL 指纹 (POST /fingerprint)

生成标准化的 SQL 模板，将字面量替换为占位符 `?`。

**请求格式:**

```json
{
  "sql": "SELECT * FROM users WHERE id = 123 AND name = 'John' AND age IN (25, 30, 35, 40, 45)",
  "dialect": "mysql",
  "max_in_values": 3
}
```

**参数说明：**
- `sHTTP API 示例

#### 使用 curl

```bash
# 解析 SQL（使用默认方言）
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1"}'

# 使用 MySQL 方言
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1", "dialect": "mysql"}'

# 生成 SQL 指纹
curl -X POST http://127.0.0.1:3000/fingerprint \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 123 AND age IN (25,30,35,40)", "dialect": "mysql", "max_in_values": 2}'
```

#### 使用 Python

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

# 生成 SQL 指纹
fingerprint_response = requests.post(
    "http://127.0.0.1:3000/fingerprint",
    json={
        "sql": "SELECT * FROM users WHERE id = 123 AND age IN (25,30,35,40)",
        "dialect": "mysql",
        "max_in_values": 2
    }
)

fingerprint_data = fingerprint_response.json()
print(f"Fingerprint: {fingerprint_data['fingerprint']}")

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

## gRPC API

### gRPC 方法

#### 1. ParseSql

解析 SQL 为 AST。

**请求:**
```protobuf
message ParseSqlRequest {
  string sql = 1;
  string dialect = 2;
  bool no_cache = 3;
}
```

**响应:**
```protobuf
message ParseSqlResponse {
  oneof result {
    ParseSuccess success = 1;
    ParseError error = 2;
  }
}
```

#### 2. GenerateFingerprint

生成 SQL 指纹。

**请求:**
```protobuf
message FingerprintRequest {
  string sql = 1;
  string dialect = 2;
  uint32 max_in_values = 3;
}
```

**响应:**
```protobuf
message FingerprintResponse {
  oneof result {
    FingerprintSuccess success = 1;
    FingerprintError error = 2;
  }
}
```

#### 3. HealthCheck

健康检查。

**请求:**
```protobuf
message HealthCheckRequest {}
```

**响应:**
```protobuf
message HealthCheckResponse {
  string status = 1;
  string version = 2;
}
```

### gRPC 客户端示例

#### 使用 grpcurl

```bash
# 安装 grpcurl
# Windows: scoop install grpcurl
# macOS: brew install grpcurl

# 列出服务
grpcurl -plaintext 127.0.0.1:50051 list

# Health Check
grpcurl -plaintext -d '{}' 127.0.0.1:50051 sql_parser.SqlParserService/HealthCheck

# Parse SQL
grpcurl -plaintext -d '{
  "sql": "SELECT * FROM users WHERE id = 123",
  "dialect": "mysql",
  "no_cache": false
}' 127.0.0.1:50051 sql_parser.SqlParserService/ParseSql

# Generate Fingerprint
grpcurl -plaintext -d '{
  "sql": "SELECT * FROM users WHERE id = 123 AND age IN (25,30,35,40)",
  "dialect": "mysql",
  "max_in_values": 2
}' 127.0.0.1:50051 sql_parser.SqlParserService/GenerateFingerprint
```

#### 客户端代码示例

查看以下文件获取完整的客户端示例：
- PowerShell: `test_grpc.ps1`
- Python: `test_grpc_client.py`

## 测试

### 单元测试

项目包含 12 个单元测试，覆盖 SQL 指纹功能：

```bash
cargo test
```

测试覆盖：
- ✅ 基本 SELECT 语句
- ✅ IN 子句限制
- ✅ UPDATE/DELETE/INSERT 语句
- ✅ 复杂 JOIN 查询
- ✅ BETWEEN 子句
- ✅ NULL 值保留
- ✅ CASE 表达式
- ✅ SQL 规范化

### API 测试

- **HTTP API**: 使用 `test_fingerprint_all.ps1` 测试脚本
- **gRPC API**: 使用 `test_grpc.ps1` 测试脚本或 `test_grpc_client.py` Python客户端

## 性能特性

- **异步处理**: 基于 Tokio 异步运行时，支持高并发
- **高性能缓存**: Moka 提供线程安全的高性能并发访问
- **双协议支持**: HTTP 和 gRPC 同时运行，互不干扰
- **性能指标**: 
  - 缓存命中: ~0.05-0.2ms
  - 缓存未命中: ~0.5-2ms（取决于 SQL 复杂度）
  - 指纹生成: ~0.05-0.2ms
  - 每秒可处理数千个请求
- **内存效率**: 可配置的缓存容量和 TTL
- **零拷贝**: gRPC 使用 Protocol Buffers 提供高效序列化

## 缓存机制

- **缓存键**: (SQL 语句, 方言) 组合
- **默认容量**: 10,000 条记录（可通过 `--cache-max-capacity` 配置）
- **默认过期时间**: 1 小时（可通过 `--cache-ttl` 配置）
- **缓存指示**: 响应中的 `cached` 字段表示是否命中缓存

相同的 SQL 语句和方言组合会被缓存，提高重复查询的性能。从缓存返回的请求通常在 0.1-0.5ms 内完成，而新解析的请求可能需要 1-5ms。

## Docker 支持

### 使用 docker-compose（推荐）

```bash
docker-compose up -d
```

### 使用 docker 命令

```bash
# 构建镜像
docker build -t sql-ast-api .

# 运行容器
docker run -d -p 3000:3000 -p 50051:50051 sql-ast-api

# 使用自定义配置
docker run -d \
  -p 8080:8080 \
  -p 50052:50052 \
  sql-ast-api \
  --host 0.0.0.0 \
  --port 8080 \
  --grpc-port 50052
```

详细的 Docker 部署指南请查看 [DOCKER.md](DOCKER.md)

## 开发

### 运行开发服务器

```bash
cargo run
```

### 运行测试

```bash
# 运行单元测试
cargo test

# 运行 HTTP API 测试
.\test_fingerprint_all.ps1

# 运行 gRPC 测试（需要 grpcurl）
.\test_grpc.ps1
```

### 格式化代码

```bash
cargo fmt
```

### 检查代码

```bash
cargo clippy
```

### 构建 release 版本

```bash
cargo build --release
```

## 项目结构

```
sql-ast-api/
├── src/
│   └── main.rs              # 主程序（HTTP + gRPC 服务）
├── proto/
│   └── sql_parser.proto     # gRPC 服务定义
├── static/
│   └── index.html           # Web 前端界面
├── build.rs                 # proto 编译脚本
├── Cargo.toml               # Rust 依赖配置
├── Dockerfile               # Docker 镜像定义
├── docker-compose.yml       # Docker Compose 配置
├── README.md                # 项目文档
├── FINGERPRINT.md           # SQL 指纹功能文档
├── test_grpc.ps1           # gRPC 测试脚本
├── test_grpc_client.py     # Python gRPC 客户端示例
└── test_fingerprint_all.ps1 # HTTP API 测试脚本
```

## 相关文档

- [SQL 指纹功能详解](FINGERPRINT.md)
- [Docker 部署指南](DOCKER.md)
- [性能测试报告](PERFORMANCE.md)
- [更新日志](CHANGELOG.md)

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

