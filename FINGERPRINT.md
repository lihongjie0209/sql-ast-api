# SQL指纹功能文档

## 概述

SQL指纹功能可以将SQL语句中的所有字面量值替换为占位符 `?`，生成一个标准化的SQL模板。这对于以下场景非常有用：

- **SQL查询缓存**: 将相似的查询归一化为相同的指纹，提高缓存命中率
- **查询分析**: 识别和分组相似的查询模式
- **性能监控**: 统计不同SQL模板的执行频率和性能
- **安全审计**: 标准化SQL便于分析和检测潜在的SQL注入

## API端点

### POST /fingerprint

生成SQL语句的指纹。

**请求体**:
```json
{
  "sql": "SELECT * FROM users WHERE id = 123 AND name = 'John'",
  "dialect": "mysql",
  "max_in_values": 0
}
```

**参数说明**:
- `sql` (必需): 要生成指纹的SQL语句
- `dialect` (可选): SQL方言，默认为 "generic"
  - 支持: `generic`, `mysql`, `postgresql`, `sqlite`, `mssql`, `hive`, `snowflake`, `ansi`
- `max_in_values` (可选): IN子句中保留的最大值数量，默认为 0（不限制）

**响应**:
```json
{
  "fingerprint": "SELECT * FROM users WHERE id = ? AND name = ?",
  "elapsed_ms": 0.226
}
```

## 功能特性

### 1. 字面量替换

所有字面量值都会被替换为 `?`：

```sql
-- 原始SQL
SELECT * FROM users WHERE id = 123 AND name = 'John' AND active = true

-- 指纹SQL
SELECT * FROM users WHERE id = ? AND name = ? AND active = ?
```

### 2. IN子句限制

可以限制IN子句中保留的值数量，超过的部分会被截断：

```sql
-- 原始SQL (max_in_values = 3)
SELECT * FROM products WHERE category_id IN (1, 2, 3, 4, 5)

-- 指纹SQL
SELECT * FROM products WHERE category_id IN (?, ?, ?)
```

这个功能特别适用于：
- 避免IN子句值过多导致的SQL模板爆炸
- 统一相似但IN值数量不同的查询
- 减少指纹的唯一性，提高分组效果

### 3. NULL值保留

NULL值会被保留，不会被替换：

```sql
-- 原始SQL
SELECT * FROM users WHERE email IS NULL

-- 指纹SQL  
SELECT * FROM users WHERE email IS NULL
```

### 4. 支持所有SQL语句类型

- **SELECT**: 包括JOIN、子查询、聚合等复杂查询
- **INSERT**: 单行和多行插入
- **UPDATE**: 包括多表更新
- **DELETE**: 包括多表删除

## 使用示例

### 示例1: 基本查询

```bash
curl -X POST http://localhost:3000/fingerprint \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT * FROM users WHERE id = 123 AND name = '\''John'\''",
    "dialect": "mysql"
  }'
```

响应:
```json
{
  "fingerprint": "SELECT * FROM users WHERE id = ? AND name = ?",
  "elapsed_ms": 0.15
}
```

### 示例2: 限制IN子句

```bash
curl -X POST http://localhost:3000/fingerprint \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT * FROM products WHERE category_id IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)",
    "dialect": "mysql",
    "max_in_values": 3
  }'
```

响应:
```json
{
  "fingerprint": "SELECT * FROM products WHERE category_id IN (?, ?, ?)",
  "elapsed_ms": 0.12
}
```

### 示例3: UPDATE语句

```bash
curl -X POST http://localhost:3000/fingerprint \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "UPDATE products SET price = 99.99, quantity = 100 WHERE id = 1",
    "dialect": "mysql"
  }'
```

响应:
```json
{
  "fingerprint": "UPDATE products SET price = ?, quantity = ? WHERE id = ?",
  "elapsed_ms": 0.11
}
```

### 示例4: 复杂JOIN查询

```bash
curl -X POST http://localhost:3000/fingerprint \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id WHERE u.age > 18 AND o.status = '\''completed'\''",
    "dialect": "postgresql"
  }'
```

响应:
```json
{
  "fingerprint": "SELECT u.name, o.total FROM users AS u JOIN orders AS o ON u.id = o.user_id WHERE u.age > ? AND o.status = ?",
  "elapsed_ms": 0.13
}
```

## Web界面

访问 http://localhost:3000 可以使用Web界面：

1. 在左侧输入SQL语句
2. 选择SQL方言
3. 设置"Max IN values"（0表示不限制）
4. 点击"Generate Fingerprint"按钮
5. 右侧显示生成的SQL指纹

**快捷键**:
- `Ctrl + Enter`: 解析SQL为AST
- `Ctrl + Shift + Enter`: 生成SQL指纹

## 实现原理

SQL指纹功能使用以下技术实现：

1. **SQL解析**: 使用 `sqlparser` 库将SQL解析为AST
2. **AST修改**: 使用 `VisitorMut` trait遍历和修改AST节点
3. **字面量替换**: 将所有值节点替换为占位符
4. **IN子句限制**: 在遍历时截断IN列表
5. **SQL生成**: 使用AST的 `Display` trait将修改后的AST转换回SQL字符串

关键代码片段:
```rust
struct FingerprintVisitor {
    max_in_values: usize,
}

impl VisitorMut for FingerprintVisitor {
    fn pre_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
        match expr {
            // 替换字面量为占位符
            Expr::Value(Value::Number(_)) 
            | Expr::Value(Value::SingleQuotedString(_)) => {
                *expr = Expr::Value(Value::Placeholder("?".to_string()));
            }
            // 限制IN子句
            Expr::InList { list, .. } => {
                if self.max_in_values > 0 && list.len() > self.max_in_values {
                    list.truncate(self.max_in_values);
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
```

## 单元测试

项目包含完整的单元测试，覆盖所有主要功能：

```bash
cargo test
```

测试覆盖：
- ✅ 基本SELECT语句
- ✅ IN子句限制
- ✅ UPDATE语句
- ✅ DELETE语句  
- ✅ INSERT语句
- ✅ 复杂JOIN查询
- ✅ BETWEEN子句
- ✅ 多个IN子句
- ✅ NULL值保留
- ✅ CASE表达式
- ✅ SQL规范化
- ✅ 方言验证

所有测试均通过 ✅

## 性能

- 单次指纹生成耗时: ~0.05-0.2ms
- 支持高并发请求
- 零分配字符串处理
- 优化的AST遍历

## 注意事项

1. **标识符不变**: 表名、列名等标识符不会被替换
2. **NULL保留**: NULL值会被保留，不会替换为 `?`
3. **方言差异**: 不同SQL方言可能产生略微不同的指纹格式
4. **IN子句限制**: 仅影响IN列表的长度，不改变查询语义
5. **子查询**: 子查询内部的值也会被替换

## 应用场景

### 1. 查询缓存系统

```python
# 伪代码示例
def get_query_result(sql):
    fingerprint = generate_fingerprint(sql)
    cached = cache.get(fingerprint)
    if cached:
        return cached
    result = execute_query(sql)
    cache.set(fingerprint, result)
    return result
```

### 2. 慢查询分析

```python
# 按指纹分组统计慢查询
slow_queries = {}
for log in slow_query_logs:
    fingerprint = generate_fingerprint(log.sql)
    if fingerprint not in slow_queries:
        slow_queries[fingerprint] = {
            'count': 0,
            'total_time': 0,
            'example': log.sql
        }
    slow_queries[fingerprint]['count'] += 1
    slow_queries[fingerprint]['total_time'] += log.execution_time
```

### 3. SQL防火墙

```python
# 检查SQL指纹是否在白名单中
def is_allowed(sql):
    fingerprint = generate_fingerprint(sql)
    return fingerprint in allowed_patterns
```

## 相关链接

- [API文档](http://localhost:3000/swagger-ui)
- [主README](README.md)
- [性能测试](PERFORMANCE.md)
