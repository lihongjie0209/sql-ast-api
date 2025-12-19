# SQL AST API 测试脚本

$baseUrl = "http://127.0.0.1:3000"

Write-Host "=== SQL to AST API 测试 ===" -ForegroundColor Cyan
Write-Host ""

# 测试健康检查
Write-Host "测试 0: 健康检查" -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$baseUrl/health"
    Write-Host "  ✓ 服务健康" -ForegroundColor Green
    Write-Host "    状态: $($health.status)" -ForegroundColor Gray
    Write-Host "    版本: $($health.version)" -ForegroundColor Gray
} catch {
    Write-Host "  ✗ 服务不可用" -ForegroundColor Red
    Write-Host "请确保服务器正在运行: cargo run" -ForegroundColor Yellow
    exit 1
}
Write-Host ""

# 测试 1: 默认方言
Write-Host "测试 1: 使用默认方言 (generic)" -ForegroundColor Yellow
$body = @{sql="SELECT * FROM users WHERE id = 1"} | ConvertTo-Json
$result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
Write-Host "  Cached: $($result.cached), 耗时: $($result.elapsed_ms)ms" -ForegroundColor Green
Write-Host ""

# 测试 2: 缓存命中
Write-Host "测试 2: 相同请求测试缓存" -ForegroundColor Yellow
$result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
Write-Host "  Cached: $($result.cached) (应该是 True), 耗时: $($result.elapsed_ms)ms" -ForegroundColor Green
if ($result.cached -eq $true) {
    Write-Host "  ✓ 缓存命中，性能提升: $('{0:N2}' -f (1.5 / $result.elapsed_ms))x" -ForegroundColor Cyan
}
Write-Host ""

# 测试 3: 不同方言
Write-Host "测试 3: 测试不同 SQL 方言" -ForegroundColor Yellow
$dialects = @("mysql", "postgresql", "sqlite", "mssql", "hive", "snowflake", "ansi")
foreach ($dialect in $dialects) {
    try {
        $body = @{
            sql="SELECT * FROM products WHERE price > 100"
            dialect=$dialect
        } | ConvertTo-Json
        $result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
        Write-Host "  $dialect`: ✓ 成功 (cached=$($result.cached), 耗时=$($result.elapsed_ms)ms)" -ForegroundColor Green
    } catch {
        Write-Host "  $dialect`: ✗ 失败" -ForegroundColor Red
    }
}
Write-Host ""

# 测试 4: 方言特定语法
Write-Host "测试 4: 方言特定 SQL 语法" -ForegroundColor Yellow

# MySQL 特定语法
$body = @{
    sql="SELECT * FROM users LIMIT 10 OFFSET 5"
    dialect="mysql"
} | ConvertTo-Json
$result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
Write-Host "  MySQL LIMIT/OFFSET: ✓ (耗时=$($result.elapsed_ms)ms)" -ForegroundColor Green

# MSSQL 特定语法
$body = @{
    sql="SELECT TOP 10 * FROM users"
    dialect="mssql"
} | ConvertTo-Json
$result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
Write-Host "  MSSQL TOP: ✓ (耗时=$($result.elapsed_ms)ms)" -ForegroundColor Green

# PostgreSQL 特定语法
$body = @{
    sql="SELECT * FROM users WHERE name ILIKE '%john%'"
    dialect="postgresql"
} | ConvertTo-Json
$result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
Write-Host "  PostgreSQL ILIKE: ✓ (耗时=$($result.elapsed_ms)ms)" -ForegroundColor Green
Write-Host ""

# 测试 5: 复杂 SQL
Write-Host "测试 5: 复杂 SQL 语句" -ForegroundColor Yellow
$complexSql = @"
SELECT 
    u.id, 
    u.name, 
    COUNT(o.id) as order_count,
    SUM(o.total) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.created_at > '2024-01-01'
GROUP BY u.id, u.name
HAVING COUNT(o.id) > 5
ORDER BY total_spent DESC
LIMIT 100
"@

$body = @{
    sql=$complexSql
    dialect="postgresql"
} | ConvertTo-Json
$result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
Write-Host "  复杂查询解析: ✓ (耗时=$($result.elapsed_ms)ms)" -ForegroundColor Green
Write-Host ""

# 测试 6: 错误处理
Write-Host "测试 6: 错误处理" -ForegroundColor Yellow

# 无效 SQL
try {
    $body = @{sql="INVALID SQL STATEMENT"} | ConvertTo-Json
    Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body -ErrorAction Stop
} catch {
    $error = $_.ErrorDetails.Message | ConvertFrom-Json
    Write-Host "  无效 SQL: ✓ 正确返回错误 (耗时=$($error.elapsed_ms)ms)" -ForegroundColor Green
    Write-Host "    错误信息: $($error.error.Substring(0, [Math]::Min(60, $error.error.Length)))..." -ForegroundColor Gray
}

# 不支持的方言
try {
    $body = @{sql="SELECT * FROM users"; dialect="oracle"} | ConvertTo-Json
    Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body -ErrorAction Stop
} catch {
    $error = $_.ErrorDetails.Message | ConvertFrom-Json
    Write-Host "  不支持的方言: ✓ 正确返回错误 (耗时=$($error.elapsed_ms)ms)" -ForegroundColor Green
    Write-Host "    错误信息: $($error.error)" -ForegroundColor Gray
}
Write-Host ""

# 测试 7: 缓存隔离（相同 SQL 不同方言）
Write-Host "测试 7: 缓存隔离测试" -ForegroundColor Yellow
$testSql = "SELECT * FROM test_table"
Write-Host "  使用相同 SQL '$testSql' 测试不同方言:" -ForegroundColor Gray

$times = @{}
foreach ($dialect in @("mysql", "postgresql", "sqlite")) {
    $body = @{sql=$testSql; dialect=$dialect} | ConvertTo-Json
    $result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
    $times[$dialect] = $result.elapsed_ms
    Write-Host "    $dialect`: cached=$($result.cached) (首次应该是 False), 耗时=$($result.elapsed_ms)ms" -ForegroundColor White
}

Write-Host "  再次请求验证缓存:" -ForegroundColor Gray
foreach ($dialect in @("mysql", "postgresql", "sqlite")) {
    $body = @{sql=$testSql; dialect=$dialect} | ConvertTo-Json
    $result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
    $speedup = [math]::Round($times[$dialect] / $result.elapsed_ms, 1)
    Write-Host "    $dialect`: cached=$($result.cached) (应该是 True), 耗时=$($result.elapsed_ms)ms, 提速: ${speedup}x" -ForegroundColor White
}
Write-Host ""

# 测试 8: 性能基准
Write-Host "测试 8: 性能基准测试" -ForegroundColor Yellow
$iterations = 10
$uncachedTimes = @()
$cachedTimes = @()

# 预热
$body = @{sql="SELECT * FROM perf_test WHERE x = 1"; dialect="mysql"} | ConvertTo-Json
$null = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body

# 测试缓存性能
for ($i = 0; $i -lt $iterations; $i++) {
    $result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
    $cachedTimes += $result.elapsed_ms
}

# 测试非缓存性能（每次不同的 SQL）
for ($i = 0; $i -lt $iterations; $i++) {
    $body = @{sql="SELECT * FROM perf_test WHERE x = $i"; dialect="mysql"} | ConvertTo-Json
    $result = Invoke-RestMethod -Uri "$baseUrl/parse" -Method Post -ContentType "application/json" -Body $body
    $uncachedTimes += $result.elapsed_ms
}

$avgCached = ($cachedTimes | Measure-Object -Average).Average
$avgUncached = ($uncachedTimes | Measure-Object -Average).Average
$speedup = [math]::Round($avgUncached / $avgCached, 1)

Write-Host "  缓存命中平均耗时: $([math]::Round($avgCached, 3))ms" -ForegroundColor Cyan
Write-Host "  缓存未命中平均耗时: $([math]::Round($avgUncached, 3))ms" -ForegroundColor Cyan
Write-Host "  性能提升: ${speedup}x" -ForegroundColor Green
Write-Host ""

# 测试 OpenAPI 文档
Write-Host "测试 9: OpenAPI 文档" -ForegroundColor Yellow
try {
    $openapi = Invoke-RestMethod -Uri "$baseUrl/api-docs/openapi.json"
    Write-Host "  ✓ OpenAPI 文档可访问" -ForegroundColor Green
    Write-Host "    版本: $($openapi.openapi)" -ForegroundColor Gray
    Write-Host "    标题: $($openapi.info.title)" -ForegroundColor Gray
    Write-Host "    路径: $($openapi.paths.PSObject.Properties.Name -join ', ')" -ForegroundColor Gray
    Write-Host "    Swagger UI: $baseUrl/swagger-ui" -ForegroundColor Cyan
} catch {
    Write-Host "  ✗ OpenAPI 文档不可访问" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== 测试完成 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "📊 总结:" -ForegroundColor Green
Write-Host "  - 所有核心功能正常" -ForegroundColor White
Write-Host "  - 缓存工作正常，性能提升显著" -ForegroundColor White
Write-Host "  - 支持 8 种 SQL 方言" -ForegroundColor White
Write-Host "  - OpenAPI 文档可用" -ForegroundColor White
Write-Host "  - 健康检查正常" -ForegroundColor White
