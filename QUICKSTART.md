# SQL to AST API - 快速开始

## 启动服务器

### 默认配置
```bash
cargo run
```
服务器将在 `http://127.0.0.1:3000` 启动

### 自定义配置
```bash
# 自定义端口
cargo run -- --port 8080

# 自定义缓存配置
cargo run -- --cache-max-capacity 5000 --cache-ttl 1800

# 监听所有网络接口
cargo run -- --host 0.0.0.0 --port 8080

# 完整配置示例
cargo run -- --host 0.0.0.0 --port 8080 --cache-max-capacity 20000 --cache-ttl 7200
```

### 查看帮助
```bash
cargo run -- --help
```

## 访问 API 文档

启动服务器后，在浏览器中打开：

- **Swagger UI**: http://127.0.0.1:3000/swagger-ui
- **OpenAPI JSON**: http://127.0.0.1:3000/api-docs/openapi.json

Swagger UI 提供了交互式 API 文档，你可以直接在浏览器中测试 API。

## 快速测试

### 1. 健康检查
```bash
curl http://127.0.0.1:3000/health
```

### 2. 解析 SQL
```bash
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1"}'
```

### 3. 使用特定方言
```bash
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users LIMIT 10", "dialect": "mysql"}'
```

### 4. 运行完整测试套件
```powershell
.\test_api.ps1
```

## 支持的 SQL 方言

- `generic` - 通用 SQL（默认）
- `mysql` - MySQL
- `postgresql` 或 `postgres` - PostgreSQL
- `sqlite` - SQLite
- `hive` - Apache Hive
- `snowflake` - Snowflake
- `mssql` 或 `sqlserver` - Microsoft SQL Server
- `ansi` - ANSI SQL

## 响应字段说明

### 成功响应
```json
{
  "ast": { ... },          // 解析后的 AST
  "cached": false,         // 是否从缓存获取
  "elapsed_ms": 1.47       // 请求处理耗时（毫秒）
}
```

### 错误响应
```json
{
  "error": "...",          // 错误信息
  "elapsed_ms": 0.18       // 请求处理耗时（毫秒）
}
```

## 性能指标

### 典型响应时间
- **缓存命中**: 0.05-0.2ms
- **缓存未命中**: 0.2-2ms（简单查询）
- **复杂查询**: 0.5-5ms

### 缓存效果
测试显示缓存可以提供 3-30 倍的性能提升，具体取决于查询复杂度。

## 配置参数详解

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--host` | 监听地址 | 127.0.0.1 |
| `--port` / `-p` | 监听端口 | 3000 |
| `--cache-max-capacity` | 缓存最大条目数 | 10000 |
| `--cache-ttl` | 缓存过期时间（秒） | 3600 |

## 使用场景

### 1. SQL 语法验证
快速验证 SQL 语句是否符合特定方言的语法规范。

### 2. SQL 分析工具
构建 SQL 分析、重写、优化工具的基础。

### 3. 数据库迁移
分析源数据库的 SQL，生成目标数据库兼容的 AST。

### 4. 教育工具
帮助学习者理解 SQL 语句的结构。

### 5. IDE 插件
为代码编辑器提供 SQL 语法分析能力。

## 部署建议

### 开发环境
```bash
cargo run
```

### 生产环境
```bash
# 编译优化版本
cargo build --release

# 运行
./target/release/sql-ast-api --host 0.0.0.0 --port 8080 --cache-max-capacity 50000
```

### Docker 部署
```bash
# 构建镜像
docker build -t sql-ast-api .

# 运行容器
docker run -d -p 8080:8080 \
  sql-ast-api \
  --host 0.0.0.0 \
  --port 8080 \
  --cache-max-capacity 50000
```

### 反向代理配置（Nginx）
```nginx
upstream sql_ast_api {
    server 127.0.0.1:3000;
}

server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://sql_ast_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

## 监控与维护

### 健康检查
设置定期健康检查：
```bash
*/1 * * * * curl -f http://127.0.0.1:3000/health || alert
```

### 性能监控
关注以下指标：
- 平均响应时间 (`elapsed_ms`)
- 缓存命中率 (`cached: true/false`)
- 错误率（HTTP 4xx 响应）

### 日志
服务器会在控制台输出启动信息，生产环境建议重定向到日志文件：
```bash
./sql-ast-api --port 8080 2>&1 | tee -a logs/api.log
```

## 故障排查

### 服务无法启动
- 检查端口是否被占用
- 验证主机地址格式是否正确
- 确保有足够的系统资源

### 响应时间过长
- 检查缓存配置是否合理
- 考虑增加 `--cache-max-capacity`
- 检查 SQL 复杂度

### 内存使用过高
- 降低 `--cache-max-capacity`
- 减少 `--cache-ttl`
- 监控缓存命中率

## 更多信息

- 完整文档：README.md
- 测试脚本：test_api.ps1
- 源代码：src/main.rs
