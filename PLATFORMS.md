# 支持的平台和架构

## 预编译二进制文件

SQL to AST API 为以下平台和架构提供预编译的二进制文件：

### Linux

| 架构 | 目标 | 文件名 | 说明 |
|------|------|--------|------|
| x86_64 | `x86_64-unknown-linux-gnu` | `sql-ast-api-linux-x86_64.tar.gz` | 标准 Linux (glibc) |
| x86_64 | `x86_64-unknown-linux-musl` | `sql-ast-api-linux-x86_64-musl.tar.gz` | 静态链接 (适用于 Alpine) |
| ARM64 | `aarch64-unknown-linux-gnu` | `sql-ast-api-linux-aarch64.tar.gz` | ARM64 Linux (glibc) |
| ARM64 | `aarch64-unknown-linux-musl` | `sql-ast-api-linux-aarch64-musl.tar.gz` | ARM64 静态链接 |

**Linux 推荐：**
- 普通 Linux 发行版（Ubuntu, Debian, CentOS 等）：使用 `-gnu` 版本
- Alpine Linux 或需要静态链接：使用 `-musl` 版本
- ARM 服务器（如 AWS Graviton）：使用 `aarch64` 版本

### Windows

| 架构 | 目标 | 文件名 | 说明 |
|------|------|--------|------|
| x86_64 | `x86_64-pc-windows-msvc` | `sql-ast-api-windows-x86_64.exe.zip` | Windows 64位 (Intel/AMD) |
| ARM64 | `aarch64-pc-windows-msvc` | `sql-ast-api-windows-aarch64.exe.zip` | Windows ARM64 (Surface Pro X 等) |

**Windows 推荐：**
- 标准 PC：使用 `x86_64` 版本
- ARM 设备（Surface Pro X, Windows Dev Kit）：使用 `aarch64` 版本

### macOS

| 架构 | 目标 | 文件名 | 说明 |
|------|------|--------|------|
| x86_64 | `x86_64-apple-darwin` | `sql-ast-api-macos-x86_64.tar.gz` | Intel Mac |
| ARM64 | `aarch64-apple-darwin` | `sql-ast-api-macos-aarch64.tar.gz` | Apple Silicon (M1/M2/M3/M4) |

**macOS 推荐：**
- Intel Mac：使用 `x86_64` 版本
- Apple Silicon Mac：使用 `aarch64` 版本（原生性能更好）
- 注意：`aarch64` 版本也可以在 Intel Mac 上运行（通过 Rosetta 2）

### Docker

| 格式 | 文件名 | 说明 |
|------|--------|------|
| Docker Image | `sql-ast-api-docker-{version}.tar.gz` | 多架构 Docker 镜像 |

## 如何选择版本

### 1. 查看系统架构

**Linux/macOS:**
```bash
uname -m
```
输出：
- `x86_64` 或 `amd64` → 使用 x86_64 版本
- `aarch64` 或 `arm64` → 使用 aarch64/ARM64 版本

**Windows PowerShell:**
```powershell
$env:PROCESSOR_ARCHITECTURE
```
输出：
- `AMD64` → 使用 x86_64 版本
- `ARM64` → 使用 aarch64 版本

### 2. 查看 Linux 发行版类型

```bash
ldd --version
```
输出包含：
- `GLIBC` → 使用 `-gnu` 版本
- `musl` → 使用 `-musl` 版本

Alpine Linux 用户始终使用 `-musl` 版本。

## 下载和安装

### 自动下载脚本

**Linux/macOS:**
```bash
# 自动检测架构并下载
curl -fsSL https://raw.githubusercontent.com/lihongjie0209/sql-ast-api/master/install.sh | sh
```

**Windows PowerShell:**
```powershell
# 自动检测架构并下载
iwr https://raw.githubusercontent.com/lihongjie0209/sql-ast-api/master/install.ps1 | iex
```

### 手动下载

1. 访问 [Releases 页面](https://github.com/lihongjie0209/sql-ast-api/releases)
2. 下载对应平台的文件
3. 解压并运行

**Linux/macOS:**
```bash
# 下载
wget https://github.com/lihongjie0209/sql-ast-api/releases/latest/download/sql-ast-api-linux-x86_64.tar.gz

# 解压
tar xzf sql-ast-api-linux-x86_64.tar.gz

# 运行
./sql-ast-api
```

**Windows:**
```powershell
# 下载（使用浏览器或 PowerShell）
# 解压 zip 文件
# 运行
.\sql-ast-api.exe
```

## 从源码构建

如果你的平台不在上述列表中，可以从源码构建：

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆仓库
git clone https://github.com/lihongjie0209/sql-ast-api.git
cd sql-ast-api

# 构建
cargo build --release

# 二进制文件位于
# target/release/sql-ast-api
```

### 交叉编译

使用 [cross](https://github.com/cross-rs/cross) 进行交叉编译：

```bash
# 安装 cross
cargo install cross --git https://github.com/cross-rs/cross

# 为其他平台构建
cross build --release --target aarch64-unknown-linux-gnu
```

## 性能对比

| 平台 | 架构 | 相对性能 | 说明 |
|------|------|----------|------|
| Linux | x86_64 (glibc) | 100% | 基准 |
| Linux | x86_64 (musl) | 95-98% | 静态链接轻微开销 |
| Linux | ARM64 | 90-95% | 取决于 CPU |
| Windows | x86_64 | 98-100% | 与 Linux 相当 |
| Windows | ARM64 | 85-90% | ARM64 on Windows |
| macOS | x86_64 | 98-100% | Intel Mac |
| macOS | ARM64 | 110-120% | M 系列芯片性能优异 |

## CI/CD 构建

所有平台的二进制文件都通过 GitHub Actions 自动构建：

- **CI 构建**: 每次 push 都会构建所有平台
- **Release 构建**: Tag 发布时自动创建 Release 并上传所有二进制文件
- **测试**: 原生平台会运行完整测试（交叉编译平台跳过测试）

查看构建状态：[GitHub Actions](https://github.com/lihongjie0209/sql-ast-api/actions)

## 支持的最低版本

- **Linux**: glibc 2.31+ (Ubuntu 20.04+, Debian 11+)
- **Windows**: Windows 10 1809+
- **macOS**: macOS 11+ (Big Sur)

## 问题反馈

如果你在某个平台上遇到问题，请：

1. 查看 [Issues](https://github.com/lihongjie0209/sql-ast-api/issues)
2. 提供系统信息：
   - 操作系统版本
   - CPU 架构
   - 错误信息

我们会尽快处理！
