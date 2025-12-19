# SQL to AST API - 前端调试页面使用指南

## 访问方式

启动服务后，在浏览器中打开：http://127.0.0.1:3000

## 页面布局

```
┌─────────────────────────────────────────────────────────┐
│  🚀 SQL to AST Parser                                   │
│  Parse SQL statements to Abstract Syntax Tree           │
├──────────────────────┬──────────────────────────────────┤
│  📝 SQL Input        │  🌳 AST Output                    │
│                      │                                   │
│  [Dialect ▼]        │  ⏱️ Time: 1.47ms                  │
│  [☐ Disable Cache]  │  💾 Cache: MISS                   │
│  [Parse SQL]        │                                   │
│  [Clear]            │                                   │
│                      │                                   │
│  SELECT * FROM...   │  {                                │
│                      │    "Query": {                     │
│                      │      "body": { ... }              │
│                      │    }                              │
│                      │  }                                │
└──────────────────────┴──────────────────────────────────┘
```

## 主要功能

### 1. SQL 编辑区（左侧）

#### 控制栏
- **Dialect 下拉框**: 选择 SQL 方言
  - Generic（通用）
  - MySQL
  - PostgreSQL
  - SQLite
  - MS SQL Server
  - Apache Hive
  - Snowflake
  - ANSI SQL

- **Disable Cache 复选框**: 
  - ☐ 未勾选：使用缓存（默认）
  - ☑ 勾选：禁用缓存，每次重新解析

- **Parse SQL 按钮**: 解析 SQL
  - 点击解析当前输入的 SQL
  - 快捷键：`Ctrl + Enter`

- **Clear 按钮**: 清空所有内容

#### 示例按钮
- **Example 1**: 简单的 SELECT 查询
- **Example 2**: 复杂的多表 JOIN 查询
- **Example 3**: INSERT 插入语句

#### 文本编辑器
- 支持多行输入
- 等宽字体显示
- 自动对焦
- 快捷键支持

### 2. AST 输出区（右侧）

#### 性能指标
- **⏱️ Time**: 请求处理耗时（毫秒）
  - 显示实际的 API 响应时间
  - 精确到小数点后 2 位

- **💾 Cache**: 缓存状态
  - `HIT`（绿色）：从缓存返回
  - `MISS`（橙色）：新解析
  - `N/A`（灰色）：错误或未知

#### 消息提示
- ✅ 成功消息（绿色背景）
  - "SQL parsed successfully!"
  
- ❌ 错误消息（红色背景）
  - SQL 语法错误
  - 不支持的方言
  - 网络错误

#### JSON 结构化显示
- **语法高亮**:
  - 键名：紫色粗体
  - 字符串：蓝色
  - 数字：深蓝色
  - 布尔值：蓝色粗体
  - null：灰色斜体
  - 括号：黑色粗体

- **折叠/展开功能**:
  - 点击 `{` 或 `[` 折叠当前对象/数组
  - 再次点击展开
  - 方便查看大型 AST 结构

## 使用示例

### 示例 1: 基本查询

1. 输入 SQL：
```sql
SELECT * FROM users WHERE id = 1
```

2. 选择方言：MySQL

3. 点击 "Parse SQL"

4. 查看结果：
   - 右侧显示完整的 AST 结构
   - 性能指标显示响应时间
   - 缓存状态显示是否命中缓存

### 示例 2: 复杂查询

1. 点击 "Example 2" 按钮

2. 自动填充复杂 SQL：
```sql
SELECT u.id, u.name, COUNT(o.id) as order_count, SUM(o.total) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.created_at > '2024-01-01'
GROUP BY u.id, u.name
HAVING COUNT(o.id) > 5
ORDER BY total_spent DESC
LIMIT 100
```

3. 解析并查看详细的 AST 结构

### 示例 3: 测试缓存

1. 输入并解析一个 SQL：
   - 第一次：Cache 显示 "MISS"（橙色）
   - 响应时间：约 1-2ms

2. 不修改 SQL，再次点击解析：
   - 第二次：Cache 显示 "HIT"（绿色）
   - 响应时间：约 0.05-0.1ms（快 10-30 倍）

### 示例 4: 禁用缓存调试

1. 勾选 "Disable Cache" 复选框

2. 多次解析同一个 SQL：
   - 每次都显示 "MISS"
   - 每次都重新解析
   - 响应时间相似

用途：
- 测试 SQL 解析器的真实性能
- 对比缓存优化效果
- 调试缓存相关问题

## 快捷键

- `Ctrl + Enter` - 解析 SQL
- 鼠标悬停 - 查看可交互元素

## 特点

### 🎨 精美设计
- 现代化渐变色主题
- 流畅的动画效果
- 响应式布局

### ⚡ 高性能
- 实时解析反馈
- 异步 API 调用
- 不阻塞 UI

### 🌳 结构化展示
- JSON 语法高亮
- 可折叠的树形结构
- 清晰的层级关系

### 📊 性能可视化
- 实时耗时显示
- 缓存命中状态
- 性能对比直观

### 🔌 离线可用
- 无外部 CDN 依赖
- 所有资源本地化
- 完全独立运行

### 📱 移动友好
- 响应式设计
- 触摸优化
- 小屏幕自适应

## 技术特性

### 前端技术
- 纯 HTML + CSS + JavaScript
- 无需构建工具
- 无外部依赖
- 嵌入到 Rust 二进制文件中

### API 集成
- RESTful API 调用
- JSON 数据交换
- 错误处理完善
- 性能监控内置

## 故障排查

### 页面无法加载
1. 检查服务是否运行
2. 访问 http://127.0.0.1:3000/health
3. 查看控制台日志

### 解析失败
1. 检查 SQL 语法是否正确
2. 确认方言选择是否匹配
3. 查看错误提示信息

### 性能异常
1. 观察缓存状态
2. 测试禁用缓存的性能
3. 检查网络延迟

## 最佳实践

1. **学习 SQL 结构**
   - 使用简单 SQL 开始
   - 逐步增加复杂度
   - 观察 AST 变化

2. **性能测试**
   - 先禁用缓存测试基线
   - 再启用缓存对比
   - 记录性能数据

3. **方言对比**
   - 同一 SQL 测试不同方言
   - 观察 AST 差异
   - 理解方言特性

4. **调试工具**
   - 使用示例快速验证
   - 保存有用的 SQL
   - 利用折叠功能导航大型 AST

## 高级用法

### 与 API 集成

前端页面实际上是调用后端 `/parse` API：

```javascript
fetch('/parse', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
    },
    body: JSON.stringify({
        sql: "SELECT * FROM users",
        dialect: "mysql",
        no_cache: false
    })
})
```

你可以在浏览器开发者工具中查看：
- Network 标签：查看请求详情
- Console 标签：查看日志信息
- Performance 标签：分析性能

### 自定义样式

前端页面的样式完全在 HTML 中定义，可以根据需要自定义：
- 修改 `static/index.html`
- 调整 CSS 样式
- 重新启动服务

## 相关文档

- 完整功能：README.md
- 快速开始：QUICKSTART.md
- Docker 部署：DOCKER.md
- API 文档：http://127.0.0.1:3000/swagger-ui
