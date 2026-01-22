# gRPC客户端测试脚本（使用grpcurl工具）

# 安装grpcurl (如果还没有安装):
# Windows: scoop install grpcurl 或 choco install grpcurl
# macOS: brew install grpcurl
# Linux: 参考 https://github.com/fullstorydev/grpcurl

$grpcUrl = "127.0.0.1:50051"

Write-Host "================================" -ForegroundColor Cyan
Write-Host "gRPC服务测试" -ForegroundColor Cyan
Write-Host "================================`n" -ForegroundColor Cyan

# 检查grpcurl是否安装
if (-not (Get-Command grpcurl -ErrorAction SilentlyContinue)) {
    Write-Host "❌ grpcurl未安装。请先安装grpcurl:" -ForegroundColor Red
    Write-Host "   Windows: scoop install grpcurl" -ForegroundColor Yellow
    Write-Host "   或访问: https://github.com/fullstorydev/grpcurl`n" -ForegroundColor Yellow
    
    Write-Host "如果没有grpcurl，可以使用以下替代方法:" -ForegroundColor Yellow
    Write-Host "1. 使用Postman (支持gRPC)" -ForegroundColor Yellow
    Write-Host "2. 使用BloomRPC" -ForegroundColor Yellow
    Write-Host "3. 使用gRPCui: grpcui -plaintext $grpcUrl`n" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ 检测到grpcurl`n" -ForegroundColor Green

# 列出可用的服务
Write-Host "📋 列出gRPC服务:" -ForegroundColor Yellow
grpcurl -plaintext $grpcUrl list
Write-Host ""

# 列出服务的方法
Write-Host "📋 列出SqlParserService的方法:" -ForegroundColor Yellow
grpcurl -plaintext $grpcUrl list sql_parser.SqlParserService
Write-Host ""

# 测试1: Health Check
Write-Host "测试1: Health Check" -ForegroundColor Yellow
grpcurl -plaintext -d '{}' $grpcUrl sql_parser.SqlParserService/HealthCheck
Write-Host ""

# 测试2: Parse SQL
Write-Host "测试2: Parse SQL" -ForegroundColor Yellow
$parseSqlRequest = @'
{
  "sql": "SELECT * FROM users WHERE id = 123",
  "dialect": "mysql",
  "no_cache": false
}
'@

grpcurl -plaintext -d $parseSqlRequest $grpcUrl sql_parser.SqlParserService/ParseSql
Write-Host ""

# 测试3: Generate Fingerprint
Write-Host "测试3: Generate Fingerprint" -ForegroundColor Yellow
$fingerprintRequest = @'
{
  "sql": "SELECT * FROM users WHERE id = 123 AND name = 'John' AND age IN (25, 30, 35, 40, 45)",
  "dialect": "mysql",
  "max_in_values": 3
}
'@

grpcurl -plaintext -d $fingerprintRequest $grpcUrl sql_parser.SqlParserService/GenerateFingerprint
Write-Host ""

# 测试4: 复杂SQL解析
Write-Host "测试4: 复杂SQL解析" -ForegroundColor Yellow
$complexSqlRequest = @'
{
  "sql": "SELECT u.name, COUNT(o.id) FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.name",
  "dialect": "postgresql",
  "no_cache": false
}
'@

grpcurl -plaintext -d $complexSqlRequest $grpcUrl sql_parser.SqlParserService/ParseSql
Write-Host ""

Write-Host "================================" -ForegroundColor Cyan
Write-Host "所有测试完成！" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Cyan
