# 01 - 开发环境搭建

## 概述

本文档指导开发者搭建 Sailfish Studio 的本地开发环境。Sailfish Studio 使用 Rust + WebAssembly 作为核心引擎，Next.js 作为前端框架，Tauri 作为桌面端框架。请严格按照以下步骤操作，确保所有工具版本符合项目要求。

> 对应开发清单：`docs/development/09-implementation-checklist.md` Section 1
> 对应需求文档：`docs/requirements/03-architecture.md` Section 3.1

## 系统要求

| 项目 | 最低要求 | 推荐配置 |
|------|---------|---------|
| 操作系统 | macOS 12+ / Windows 10+ / Ubuntu 20.04+ | macOS 14+ / Windows 11 / Ubuntu 22.04+ |
| 内存 | 16GB | 32GB |
| 磁盘 | 50GB 可用空间 | 100GB SSD |
| CPU | 4 核 | 8 核+ |

## 安装步骤

### 1. Rust (≥ 1.87，edition 2024)

Rust 是 Sailfish Studio 核心引擎的编写语言，必须首先安装。项目使用 Rust 2024 edition，需要 Rust 1.85+ 编译器，建议使用 1.87+ 以获得最佳开发体验。

```bash
# 安装 rustup（Rust 工具链管理器）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 使用 stable 通道
rustup default stable

# 添加 WebAssembly 编译目标
rustup target add wasm32-unknown-unknown

# 验证安装
rustc --version  # 应显示 ≥ 1.87
cargo --version
```

> **重要**：项目使用 `edition = "2024"`，Rust 2024 edition 于 Rust 1.85 正式发布。请确保你的编译器版本不低于此要求。

### 2. Node.js 与 pnpm (≥ 10)

前端编辑器使用 Next.js 16 构建，需要 Node.js 和 pnpm 作为包管理器。推荐使用 nvm 管理 Node.js 版本。

```bash
# 安装 nvm（Node Version Manager）
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc  # 或 source ~/.zshrc

# 安装并使用 Node.js 24 LTS
nvm install 24
nvm use 24

# 启用 pnpm（通过 corepack）
corepack enable pnpm
corepack prepare pnpm@latest --activate

# 验证安装
node --version   # 应显示 v24.x
pnpm --version   # 应显示 10.x
```

### 3. Rust 工具链

以下 Rust 工具是开发、测试和发布流程的核心依赖。请逐一安装：

```bash
# wasm-pack：将 Rust 编译为 WebAssembly 并生成 JS 绑定
cargo install wasm-pack

# wasm-bindgen-cli：Rust ↔ JS 互操作代码生成器
cargo install wasm-bindgen-cli

# cargo-nextest：下一代 Rust 测试运行器（比 cargo test 快 3-5x）
cargo install cargo-nextest

# cargo-audit：检查依赖中的安全漏洞
cargo install cargo-audit

# cargo-tarpaulin：Rust 代码覆盖率工具
cargo install cargo-tarpaulin

# cargo-fuzz：模糊测试框架（基于 libFuzzer）
cargo install cargo-fuzz
```

### 4. 前端工具

前端代码使用 Biome 进行格式化和 lint（替代 ESLint + Prettier），使用 Playwright 进行端到端测试。

```bash
pnpm add -g @biomejs/biome @playwright/test
```

安装 Playwright 浏览器（首次使用时）：

```bash
pnpm exec playwright install
```

### 5. Emscripten (C/C++ 扩展编译，仅 Phase 7 需要)

如果需要开发或编译 C/C++ 扩展，需要安装 Emscripten SDK。这是可选依赖，仅在开发 C/C++ 扩展时才需要。

```bash
# 克隆 emsdk
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk

# 安装并激活最新版本
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh

# 将 source 命令添加到 shell 配置文件中，避免每次手动执行
echo 'source /path/to/emsdk/emsdk_env.sh' >> ~/.bashrc
```

### 6. Tauri CLI (≥ 2.0，仅 Phase 6 需要)

桌面端应用使用 Tauri 2.0 构建。Tauri CLI 用于开发调试和打包发布。

```bash
cargo install tauri-cli
```

Tauri 2.0 还需要系统级依赖：

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# macOS
# Xcode Command Line Tools（通常已安装）
xcode-select --install

# Windows
# 需要 Visual Studio C++ Build Tools 和 WebView2（Windows 10/11 已内置）
```

### 7. LLVM (≥ 19，仅 AOT 编译器 Phase 5 需要)

AOT 编译器使用 LLVM 19+ 作为后端，将 .sfl/.sf 项目编译为原生可执行文件。这是可选依赖，仅在开发 AOT 编译器时需要。

```bash
# Ubuntu/Debian
sudo apt install llvm-19-dev libclang-19-dev

# macOS
brew install llvm@19

# 验证安装
llvm-config --version  # 应显示 19.x
```

## 克隆仓库

Sailfish Studio 采用多仓库架构，核心仓库如下。完整仓库列表请参考 `docs/development/02-project-structure.md`。

```bash
# 核心仓库（必须）
git clone https://github.com/Sailfish-Studio/sf-core.git

# 编辑器仓库（必须）
git clone https://github.com/Sailfish-Studio/sf-editor.git

# 其他仓库按需克隆
git clone https://github.com/Sailfish-Studio/sf-runtime.git
git clone https://github.com/Sailfish-Studio/sf-tools.git
git clone https://github.com/Sailfish-Studio/sf-extensions.git
git clone https://github.com/Sailfish-Studio/sf-services.git
git clone https://github.com/Sailfish-Studio/sf-aot-compiler.git
git clone https://github.com/Sailfish-Studio/sf-docs.git
```

## 验证安装

运行以下命令验证所有工具已正确安装：

```bash
# 验证 Rust 核心
cd sf-core
cargo build
cargo nextest run

# 验证前端编辑器
cd ../sf-editor
pnpm install
pnpm build
pnpm dev
```

访问 http://localhost:3000，看到 Sailfish Studio 编辑器界面即表示环境搭建成功。

## 常见问题

**Q: cargo build 失败，提示 wasm32-unknown-unknown 未安装？**
A: 运行 `rustup target add wasm32-unknown-unknown`，然后重新构建。

**Q: pnpm install 很慢？**
A: 配置国内镜像源：`pnpm config set registry https://registry.npmmirror.com`。

**Q: macOS 上 Tauri 构建失败？**
A: 确保已安装 Xcode Command Line Tools：`xcode-select --install`。
