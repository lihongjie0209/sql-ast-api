# SQL to AST API - 更新日志

## v0.1.0 - 完整功能版本

### ✨ 新增功能

#### 1. 命令行参数配置
- `--host` - 配置监听地址（默认: 127.0.0.1）
- `--port` / `-p` - 配置监听端口（默认: 3000）
- `--cache-max-capacity` - 配置缓存容量（默认: 10000）
- `--cache-ttl` - 配置缓存过期时间（默认: 3600秒）

使用示例：
```bash
cargo run -- --port 8080 --cache-max-capacity 5000 --cache-ttl 1800
```

#### 2. OpenAPI 文档支持
- 集成 utoipa 和 utoipa-swagger-ui
- 自动生成 OpenAPI 3.1.0 规范文档
- Swagger UI 交互式文档界面
- 访问路径：
  - Swagger UI: `http://127.0.0.1:3000/swagger-ui`
  - OpenAPI JSON: `http://127.0.0.1:3000/api-docs/openapi.json`

#### 3. 健康检查接口
- 新增 `GET /health` 端点
- 返回服务状态和版本信息
- 用于监控和负载均衡器健康检查

响应示例：
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

#### 4. 性能指标
所有响应现在包含详细的性能指标：

**成功响应：**
```json
{
  "ast": { ... },
  "cached": false,
  "elapsed_ms": 1.47
}
```

**错误响应：**
```json
{
  "error": "...",
  "elapsed_ms": 0.18
}
```

- `cached`: 布尔值，指示是否从缓存获取
- `elapsed_ms`: 浮点数，请求处理耗时（毫秒）

### 🚀 性能表现

测试结果显示：
- 缓存命中平均耗时: **0.05ms**
- 缓存未命中平均耗时: **0.17ms**
- 性能提升: **3-30倍**（取决于查询复杂度）

### 📦 核心功能

#### SQL 方言支持（8种）
1. Generic - 通用 SQL
2. MySQL - MySQL
3. PostgreSQL - PostgreSQL
4. SQLite - SQLite
5. Hive - Apache Hive
6. Snowflake - Snowflake
7. MSSQL - Microsoft SQL Server
8. ANSI - ANSI SQL

#### 高性能缓存
- 使用 Moka 异步缓存库
- 支持并发访问
- 可配置容量和 TTL
- 自动过期和清理

#### RESTful API
- `POST /parse` - 解析 SQL 为 AST
- `GET /health` - 健康检查
- 支持 CORS
- 完整的错误处理

### 📚 文档

项目包含完整文档：
1. **README.md** - 完整功能文档和使用指南
2. **QUICKSTART.md** - 快速开始指南
3. **test_api.ps1** - 自动化测试脚本
4. **OpenAPI 文档** - 交互式 API 文档

### 🧪 测试覆盖

`test_api.ps1` 包含全面的测试：
- ✅ 健康检查
- ✅ 默认方言解析
- ✅ 缓存功能验证
- ✅ 8种方言测试
- ✅ 方言特定语法
- ✅ 复杂 SQL 查询
- ✅ 错误处理
- ✅ 缓存隔离
- ✅ 性能基准测试
- ✅ OpenAPI 文档可访问性

### 🔧 技术栈

- **axum 0.7** - Web 框架
- **tokio 1.0** - 异步运行时
- **sqlparser 0.52** - SQL 解析器
- **moka 0.12** - 高性能缓存
- **clap 4.5** - 命令行参数解析
- **utoipa 5.0** - OpenAPI 文档生成
- **serde 1.0** - 序列化/反序列化

### 📊 使用示例

#### 基础使用
```bash
# 启动服务器
cargo run

# 解析 SQL
curl -X POST http://127.0.0.1:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users", "dialect": "mysql"}'
```

#### 自定义配置
```bash
# 生产环境配置
cargo run --release -- \
  --host 0.0.0.0 \
  --port 8080 \
  --cache-max-capacity 50000 \
  --cache-ttl 7200
```

#### PowerShell 脚本
```powershell
$body = @{
    sql = "SELECT * FROM users WHERE id = 1"
    dialect = "postgresql"
} | ConvertTo-Json

$result = Invoke-RestMethod -Uri http://127.0.0.1:3000/parse `
    -Method Post -ContentType "application/json" -Body $body

Write-Host "Cached: $($result.cached), Time: $($result.elapsed_ms)ms"
```

### 🎯 应用场景

1. **SQL 验证器** - 验证 SQL 语法正确性
2. **数据库迁移工具** - 分析和转换 SQL
3. **SQL 分析工具** - 理解 SQL 结构
4. **IDE 插件** - 提供 SQL 语法支持
5. **教育工具** - 学习 SQL 语法结构
6. **查询优化器** - 分析和优化查询

### 🚀 未来计划

- [ ] 添加更多 SQL 方言支持
- [ ] 实现 AST 到 SQL 的反向转换
- [ ] 添加 SQL 格式化功能
- [ ] 支持批量解析
- [ ] 添加更多性能指标（QPS、内存使用等）
- [ ] WebSocket 支持用于实时解析
- [ ] SQL 优化建议功能

### 📝 变更记录

#### 2024-12-19
- ✅ 添加命令行参数支持（clap）
- ✅ 集成 OpenAPI/Swagger UI 文档
- ✅ 添加健康检查接口
- ✅ 添加性能指标（elapsed_ms）
- ✅ 优化缓存逻辑
- ✅ 完善测试套件
- ✅ 更新文档

#### 初始版本
- ✅ 基础 SQL 解析功能
- ✅ 8 种 SQL 方言支持
- ✅ Moka 缓存集成
- ✅ RESTful API 接口
- ✅ CORS 支持

### 🙏 致谢

感谢以下开源项目：
- [sqlparser-rs](https://github.com/sqlparser-rs/sqlparser-rs)
- [axum](https://github.com/tokio-rs/axum)
- [moka](https://github.com/moka-rs/moka)
- [utoipa](https://github.com/juhaku/utoipa)
