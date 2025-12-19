# SQL to AST API

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com/)

A high-performance Rust HTTP API for parsing SQL statements to Abstract Syntax Tree (AST) with beautiful web UI, comprehensive documentation, and Docker support.

🌐 **[Live Demo](http://localhost:3000)** (After running locally)

## ✨ Features

- ✅ Parse SQL to AST with JSON output
- ✅ Support 8 SQL dialects (MySQL, PostgreSQL, SQLite, Hive, Snowflake, MSSQL, ANSI, Generic)
- ✅ High-performance caching with Moka (20-40% hit rate improvement)
- ✅ Beautiful web UI for debugging (offline-capable)
- ✅ OpenAPI 3.0 documentation with Swagger UI
- ✅ Health check endpoint
- ✅ Performance metrics (response time, cache status)
- ✅ No-cache mode for debugging
- ✅ Docker and docker-compose support
- ✅ Comprehensive documentation

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
docker-compose up -d
```

Visit http://localhost:3000 for the web UI.

### Using Cargo

```bash
cargo run --release
```

## 📊 Performance

| Metric | Performance |
|--------|-------------|
| Cache Hit | 0.02-0.1ms |
| Cache Miss (Simple) | 0.3-0.6ms |
| Cache Miss (Complex) | 0.5-1.5ms |
| Throughput | Thousands of requests/sec |

**Optimizations:**
- Release mode compilation with LTO
- Dialect object reuse
- SQL normalization for cache
- Zero-cost abstractions

## 📝 API Examples

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

## 🖥️ Web UI

The web interface provides:
- SQL editor with syntax highlighting
- Dialect selector
- Real-time AST visualization
- Collapsible JSON tree view
- Performance metrics
- Cache status indicators
- Built-in SQL examples

![Screenshot](docs/screenshot.png)

## 📚 Documentation

- [Quick Start Guide](QUICKSTART.md)
- [Docker Deployment](DOCKER.md)
- [Frontend Guide](FRONTEND.md)
- [Performance Optimization](PERFORMANCE.md)
- [Optimization Report](OPTIMIZATION_REPORT.md)
- [API Documentation](http://localhost:3000/swagger-ui)

## 🛠️ Configuration

```bash
sql-ast-api [OPTIONS]

Options:
  --host <HOST>                          Server host [default: 127.0.0.1]
  -p, --port <PORT>                      Server port [default: 3000]
  --cache-max-capacity <CAPACITY>        Maximum cache entries [default: 10000]
  --cache-ttl <TTL>                      Cache TTL in seconds [default: 3600]
  -h, --help                             Print help
```

## 🐳 Docker Support

```yaml
# docker-compose.yml
version: '3.8'
services:
  sql-ast-api:
    build: .
    ports:
      - "3000:3000"
    command:
      - --host
      - "0.0.0.0"
      - --cache-max-capacity
      - "50000"
```

## 🧪 Testing

```bash
# Run tests
cargo test

# Run integration tests
./test_api.ps1
```

## 📦 Tech Stack

- **Backend**: Rust (Axum, Tokio, Moka, Sqlparser)
- **Frontend**: Pure HTML/CSS/JavaScript (no dependencies)
- **Documentation**: OpenAPI 3.0 (utoipa, Swagger UI)
- **Deployment**: Docker, docker-compose

## 🔧 Development

```bash
# Build
cargo build --release

# Run with custom config
cargo run -- --port 8080 --cache-max-capacity 20000

# Format code
cargo fmt

# Lint
cargo clippy
```

## 📈 Project Status

- ✅ Core functionality complete
- ✅ Performance optimized
- ✅ Production ready
- ✅ Fully documented
- ✅ Docker support
- ✅ Comprehensive tests

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 Acknowledgments

- [sqlparser-rs](https://github.com/sqlparser-rs/sqlparser-rs) - SQL parser
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [moka](https://github.com/moka-rs/moka) - High-performance cache
- [utoipa](https://github.com/juhaku/utoipa) - OpenAPI documentation

## 📞 Support

- 📖 [Documentation](README.md)
- 🐛 [Issue Tracker](https://github.com/lihongjie0209/sql-ast-api/issues)
- 💬 [Discussions](https://github.com/lihongjie0209/sql-ast-api/discussions)

---

**Made with ❤️ using Rust**
