# SQL to AST API v0.1.0

A high-performance Rust HTTP API for parsing SQL statements to Abstract Syntax Tree (AST).

## Features

- **SQL Parsing**: Parse SQL to AST with JSON output
- **8 SQL Dialects**: MySQL, PostgreSQL, SQLite, Hive, Snowflake, MSSQL, ANSI, Generic
- **High Performance**: Optimized with release mode + LTO
  - Cache hit: 0.02-0.1ms
  - Cache miss: 0.3-1.5ms
  - 20-40% cache hit rate improvement with SQL normalization
- **Web UI**: Beautiful offline-capable debugging interface
- **OpenAPI Docs**: Swagger UI with complete API documentation
- **Docker Support**: Ready-to-use Docker and docker-compose
- **Multi-platform**: 9 platform/architecture combinations
- **Performance Metrics**: Real-time response time and cache status
- **Comprehensive Docs**: 11 detailed documentation files

## Performance

| Metric | Performance |
|--------|-------------|
| Cache Hit | 0.02-0.1ms |
| Simple Query | 0.3-0.6ms |
| Complex Query | 0.5-1.5ms |
| Throughput | Thousands req/sec |

## Supported Platforms

### Download Pre-built Binaries

**Linux:**
- `sql-ast-api-linux-x86_64.tar.gz` - Standard Linux (glibc)
- `sql-ast-api-linux-x86_64-musl.tar.gz` - Static binary (Alpine)
- `sql-ast-api-linux-aarch64.tar.gz` - ARM64 Linux
- `sql-ast-api-linux-aarch64-musl.tar.gz` - ARM64 static binary

**Windows:**
- `sql-ast-api-windows-x86_64.exe.zip` - Windows 64-bit
- `sql-ast-api-windows-aarch64.exe.zip` - Windows ARM64

**macOS:**
- `sql-ast-api-macos-x86_64.tar.gz` - Intel Mac
- `sql-ast-api-macos-aarch64.tar.gz` - Apple Silicon (M1/M2/M3)

**Docker:**
- `sql-ast-api-docker-0.1.0.tar.gz` - Docker image

See [PLATFORMS.md](https://github.com/lihongjie0209/sql-ast-api/blob/master/PLATFORMS.md) for platform selection guide.

## Quick Start

### Using Docker

```bash
docker-compose up -d
```

### Using Pre-built Binary

**Linux/macOS:**
```bash
# Download and extract
tar xzf sql-ast-api-linux-x86_64.tar.gz

# Run
./sql-ast-api
```

**Windows:**
```powershell
# Extract zip file and run
.\sql-ast-api.exe
```

Visit http://localhost:3000 for the web UI.

### Using Cargo

```bash
cargo install --git https://github.com/lihongjie0209/sql-ast-api
```

## Configuration

```bash
sql-ast-api [OPTIONS]

Options:
  --host <HOST>                  Server host [default: 127.0.0.1]
  -p, --port <PORT>              Server port [default: 3000]
  --cache-max-capacity <NUM>     Max cache entries [default: 10000]
  --cache-ttl <SECONDS>          Cache TTL [default: 3600]
```

## API Examples

### Parse SQL

```bash
curl -X POST http://localhost:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT * FROM users WHERE id = 1", "dialect": "mysql"}'
```

### Health Check

```bash
curl http://localhost:3000/health
```

## Documentation

- [README.md](https://github.com/lihongjie0209/sql-ast-api/blob/master/README.md) - Complete documentation
- [QUICKSTART.md](https://github.com/lihongjie0209/sql-ast-api/blob/master/QUICKSTART.md) - Quick start guide
- [DOCKER.md](https://github.com/lihongjie0209/sql-ast-api/blob/master/DOCKER.md) - Docker deployment
- [FRONTEND.md](https://github.com/lihongjie0209/sql-ast-api/blob/master/FRONTEND.md) - Web UI guide
- [PERFORMANCE.md](https://github.com/lihongjie0209/sql-ast-api/blob/master/PERFORMANCE.md) - Optimization guide
- [PLATFORMS.md](https://github.com/lihongjie0209/sql-ast-api/blob/master/PLATFORMS.md) - Platform support
- [API Documentation](http://localhost:3000/swagger-ui) - Interactive API docs

## Optimizations

- Release mode compilation with LTO (Link Time Optimization)
- Dialect object reuse with Arc for zero-cost sharing
- SQL normalization for better cache hit rates
- Zero-cost abstractions throughout

## What's Included

- Rust source code
- Web UI (HTML/CSS/JS - offline capable)
- Docker configuration
- GitHub Actions CI/CD
- Comprehensive documentation
- Test scripts
- Pre-built binaries for 9 platforms

## Technology Stack

- **Backend**: Rust (Axum, Tokio, Moka, Sqlparser)
- **Frontend**: Pure HTML/CSS/JavaScript (no dependencies)
- **Documentation**: OpenAPI 3.0 (utoipa, Swagger UI)
- **Deployment**: Docker, docker-compose
- **CI/CD**: GitHub Actions with multi-platform builds

## Acknowledgments

Thanks to the amazing open source projects:
- [sqlparser-rs](https://github.com/sqlparser-rs/sqlparser-rs) - SQL parser
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [moka](https://github.com/moka-rs/moka) - High-performance cache
- [utoipa](https://github.com/juhaku/utoipa) - OpenAPI documentation

## License

MIT License

---

**Made with love using Rust**
