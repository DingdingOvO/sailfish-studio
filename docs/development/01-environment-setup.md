# 01 - 开发环境搭建

## 概述

本文档指导开发者搭建 Sailfish Studio 的本地开发环境。

## 系统要求

- 操作系统: macOS 12+ / Windows 10+ / Ubuntu 20.04+
- 内存: 16GB+ 推荐
- 磁盘: 50GB+ 可用空间

## 安装步骤

### 1. Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

### 2. Node.js & pnpm

```bash
# 使用 nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 24
nvm use 24
corepack enable pnpm
```

### 3. wasm-pack

```bash
cargo install wasm-pack
```

### 4. Emscripten (用于 C/C++ 扩展编译)

```bash
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install 4.2
./emsdk activate 4.2
source ./emsdk_env.sh
```

### 5. Tauri CLI (桌面端开发)

```bash
cargo install tauri-cli
```

### 6. 其他工具

```bash
cargo install cargo-nextest cargo-audit cargo-tarpaulin cargo-fuzz
pnpm add -g @biomejs/biome @playwright/test
```

## 克隆仓库

```bash
git clone https://github.com/Sailfish-Studio/sf-core.git
git clone https://github.com/Sailfish-Studio/sf-editor.git
# ... 其他仓库
```

## 验证安装

```bash
cd sf-editor
pnpm install
pnpm build
pnpm dev
```

访问 http://localhost:3000，看到编辑器界面即表示成功。
