# SQL to AST API - Docker 部署指南

## 快速开始

### 使用 docker-compose（推荐）

```bash
# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

服务将在 `http://localhost:3000` 启动

### 使用 Docker 命令

#### 构建镜像

```bash
docker build -t sql-ast-api .
```

#### 运行容器

```bash
# 基础运行
docker run -d -p 3000:3000 sql-ast-api

# 自定义配置
docker run -d \
  -p 8080:8080 \
  --name sql-ast-api \
  sql-ast-api \
  --host 0.0.0.0 \
  --port 8080 \
  --cache-max-capacity 50000 \
  --cache-ttl 7200

# 查看日志
docker logs -f sql-ast-api

# 停止容器
docker stop sql-ast-api

# 删除容器
docker rm sql-ast-api
```

## 配置说明

### 环境变量

docker-compose.yml 中可以配置的参数：

```yaml
command: 
  - --host
  - "0.0.0.0"              # 监听地址
  - --port
  - "3000"                  # 端口
  - --cache-max-capacity
  - "10000"                 # 缓存容量
  - --cache-ttl
  - "3600"                  # 缓存过期时间（秒）
```

### 端口映射

```yaml
ports:
  - "主机端口:容器端口"
```

例如：
- `"3000:3000"` - 主机和容器都使用 3000 端口
- `"8080:3000"` - 主机使用 8080，容器使用 3000

## 健康检查

容器包含健康检查配置：

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
  interval: 30s      # 每30秒检查一次
  timeout: 3s        # 超时时间3秒
  retries: 3         # 失败重试3次
  start_period: 5s   # 启动后5秒开始检查
```

查看健康状态：

```bash
docker ps
# 或
docker inspect sql-ast-api | jq '.[0].State.Health'
```

## 访问服务

容器启动后，可以访问：

- **前端调试页面**: http://localhost:3000
- **API 文档**: http://localhost:3000/swagger-ui
- **健康检查**: http://localhost:3000/health
- **解析接口**: http://localhost:3000/parse (POST)

## 生产环境部署

### 使用自定义配置

创建 `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  sql-ast-api:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: sql-ast-api-prod
    ports:
      - "8080:8080"
    restart: always
    command: 
      - --host
      - "0.0.0.0"
      - --port
      - "8080"
      - --cache-max-capacity
      - "50000"
      - --cache-ttl
      - "7200"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

启动：

```bash
docker-compose -f docker-compose.prod.yml up -d
```

### 使用 Nginx 反向代理

创建 `nginx.conf`:

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
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support (if needed)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

使用 docker-compose 部署 Nginx + API:

```yaml
version: '3.8'

services:
  sql-ast-api:
    build: .
    container_name: sql-ast-api
    restart: always
    command: 
      - --host
      - "0.0.0.0"
      - --port
      - "3000"
      - --cache-max-capacity
      - "50000"
    networks:
      - api-network

  nginx:
    image: nginx:alpine
    container_name: nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
      - ./certs:/etc/nginx/certs
    depends_on:
      - sql-ast-api
    restart: always
    networks:
      - api-network

networks:
  api-network:
    driver: bridge
```

## 监控与日志

### 查看实时日志

```bash
docker-compose logs -f sql-ast-api
```

### 持久化日志

修改 docker-compose.yml:

```yaml
services:
  sql-ast-api:
    # ... 其他配置
    volumes:
      - ./logs:/app/logs
    command:
      - --host
      - "0.0.0.0"
      # ... 其他参数
```

### 资源使用监控

```bash
# 查看资源使用情况
docker stats sql-ast-api

# 查看详细信息
docker inspect sql-ast-api
```

## 故障排查

### 容器无法启动

```bash
# 查看错误日志
docker logs sql-ast-api

# 检查容器状态
docker ps -a

# 进入容器调试
docker exec -it sql-ast-api /bin/bash
```

### 端口冲突

如果 3000 端口已被占用：

```bash
# 使用其他端口
docker run -d -p 8080:3000 sql-ast-api
```

### 健康检查失败

```bash
# 手动测试健康检查
docker exec sql-ast-api curl -f http://localhost:3000/health

# 查看健康检查历史
docker inspect sql-ast-api | jq '.[0].State.Health.Log'
```

## 备份与恢复

### 导出镜像

```bash
docker save sql-ast-api > sql-ast-api.tar
```

### 导入镜像

```bash
docker load < sql-ast-api.tar
```

### 数据持久化

虽然此服务主要使用内存缓存，如需持久化配置或日志：

```yaml
volumes:
  - ./config:/app/config
  - ./logs:/app/logs
```

## 性能优化

### 多阶段构建优化

Dockerfile 已使用多阶段构建：
- 构建阶段：使用完整的 Rust 镜像
- 运行阶段：使用精简的 Debian 镜像
- 最终镜像大小：约 100MB

### 缓存优化

根据使用场景调整缓存参数：

- **高并发场景**: 增加 cache-max-capacity
- **内存受限**: 减少 cache-max-capacity
- **频繁变更**: 减少 cache-ttl

## 集群部署

### 使用 Docker Swarm

```bash
# 初始化 Swarm
docker swarm init

# 部署服务
docker stack deploy -c docker-compose.yml sql-ast-api-stack

# 扩展服务
docker service scale sql-ast-api-stack_sql-ast-api=3

# 查看服务
docker service ls
docker service ps sql-ast-api-stack_sql-ast-api
```

### 使用 Kubernetes

创建 `k8s-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sql-ast-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sql-ast-api
  template:
    metadata:
      labels:
        app: sql-ast-api
    spec:
      containers:
      - name: sql-ast-api
        image: sql-ast-api:latest
        ports:
        - containerPort: 3000
        args:
          - --host
          - "0.0.0.0"
          - --cache-max-capacity
          - "50000"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          limits:
            cpu: "1"
            memory: "1Gi"
          requests:
            cpu: "500m"
            memory: "512Mi"
---
apiVersion: v1
kind: Service
metadata:
  name: sql-ast-api
spec:
  selector:
    app: sql-ast-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

部署：

```bash
kubectl apply -f k8s-deployment.yaml
```

## 常用命令速查

```bash
# 构建
docker build -t sql-ast-api .

# 运行
docker run -d -p 3000:3000 sql-ast-api

# 停止
docker stop sql-ast-api

# 删除
docker rm sql-ast-api

# 查看日志
docker logs -f sql-ast-api

# 进入容器
docker exec -it sql-ast-api /bin/bash

# 查看资源
docker stats sql-ast-api

# 健康检查
docker exec sql-ast-api curl http://localhost:3000/health
```

## 更多信息

- 完整文档: README.md
- 快速开始: QUICKSTART.md
- 更新日志: CHANGELOG.md
