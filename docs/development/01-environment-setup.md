# 01 - 开发环境搭建

## 概述

本文档指导开发者搭建 Sailfish Studio 的本地开发环境。

> 对应开发清单：`docs/development/09-implementation-checklist.md` Section 1

## 系统要求

- 操作系统: macOS 12+ / Windows 10+ / Ubuntu 20.04+
- 内存: 16GB+
- 磁盘: 50GB+ 可用空间

## 安装步骤

### 1. Rust (≥ 1.95)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup target add wasm32-unknown-unknown
rustc --version  # 验证 ≥ 1.95
```

### 2. Node.js 与 pnpm (≥ 10)

```bash
# 使用 nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 24
nvm use 24
corepack enable pnpm
corepack prepare pnpm@latest --activate
```

### 3. Rust 工具链

```bash
cargo install wasm-pack wasm-bindgen-cli cargo-nextest cargo-audit cargo-tarpaulin cargo-fuzz
```

### 4. 前端工具

```bash
pnpm add -g @biomejs/biome @playwright/test
```

### 5. Emscripten (C/C++ 扩展编译，仅 Phase 7 需要)

```bash
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk && ./emsdk install 4.2 && ./emsdk activate 4.2
source ./emsdk_env.sh
```

### 6. Tauri CLI (≥ 2.0，仅 Phase 6 需要)

```bash
cargo install tauri-cli --version "^2.0"
```

### 7. LLVM (≥ 18，仅 AOT 编译器 Phase 5 需要)

```bash
# Ubuntu
sudo apt install llvm-18-dev libclang-18-dev

# macOS
brew install llvm@18
```

## 克隆仓库

```bash
# 核心仓库
git clone https://github.com/Sailfish-Studio/sf-core.git
git clone https://github.com/Sailfish-Studio/sf-editor.git
# 其他仓库按需克隆，完整列表见 docs/development/02-project-structure.md
```

## 验证安装

```bash
cd sf-core
cargo build
cargo nextest run

cd ../sf-editor
pnpm install
pnpm build
pnpm dev
```

访问 http://localhost:3000，看到编辑器界面即表示成功。
