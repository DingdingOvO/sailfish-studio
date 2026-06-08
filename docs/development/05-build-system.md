# 05 - 构建系统

## 概述

Sailfish Studio 的构建系统分为两个部分：Rust 侧使用 Cargo + wasm-pack 构建 WebAssembly 模块，前端侧使用 Next.js 16 (Turbopack) 构建编辑器 UI。桌面端使用 Tauri CLI 打包原生应用。本文档详细描述各平台的构建流程和产物。

> 对应需求文档：`docs/requirements/03-architecture.md` Section 3.1
> 对应设计文档：`docs/design/architecture/04-deployment.md`

## 构建流程概览

```
sf-core (Rust) ──→ wasm-pack build ──→ sf_vm.wasm + sf_vm.js
                                              ↓
sf-editor (Next.js) ──→ Turbopack ──→ 打包 Wasm + UI ──→ dist/
                                              ↓
sf-desktop (Tauri) ──→ cargo tauri build ──→ .dmg / .msi / .AppImage
```

## Web 端构建

### 开发模式

```bash
cd sf-editor

# 安装依赖
pnpm install

# 启动开发服务器（Turbopack 加速）
pnpm dev
```

开发服务器运行在 http://localhost:3000，支持热模块替换（HMR）。WASM 模块在开发模式下按需编译。

### 生产构建

```bash
cd sf-editor

# 构建 WASM 模块（先构建 sf-core）
cd ../sf-core
wasm-pack build sf-vm --target web --out-dir ../sf-editor/public/wasm/sf-vm
wasm-pack build sf-blocks --target web --out-dir ../sf-editor/public/wasm/sf-blocks
wasm-pack build sf-renderer --target web --out-dir ../sf-editor/public/wasm/sf-renderer

# 构建前端
cd ../sf-editor
pnpm build          # Next.js 静态导出 (output: 'export')
```

构建产物输出到 `sf-editor/dist/` 目录，为纯静态文件，可直接部署到任何静态托管服务。

### WASM 优化

```bash
# 使用 wasm-opt 优化 WASM 文件大小
wasm-opt -Os sf_vm.wasm -o sf_vm.wasm

# 或使用 wasm-pack 的 --release 模式自动优化
wasm-pack build sf-vm --target web --release
```

> **重要**：生产构建中不再使用 `wee_alloc`。Rust 默认分配器（dlmalloc）配合 `wasm-opt -Os` 已经提供更优的性能和更小的体积。`wee_alloc` 已停止维护且存在内存泄漏问题。

## 桌面端构建

### 开发模式

```bash
cd sf-editor

# 启动 Tauri 开发模式（同时启动前端开发服务器和 Tauri 后端）
cargo tauri dev
```

### 生产构建

```bash
cd sf-editor

# 构建桌面应用
cargo tauri build

# 构建产物位于：
# macOS: target/release/bundle/dmg/Sailfish Studio.dmg
# Windows: target/release/bundle/msi/Sailfish Studio.msi
# Linux: target/release/bundle/appimage/sailfish-studio.AppImage
```

### 代码签名

桌面端应用需要代码签名才能分发：

- **macOS**：Developer ID 证书 + 公证 (notarization)
- **Windows**：EV Code Signing Certificate
- **Linux**：GPG 签名

详见 `docs/operations/01-deployment-guide.md`。

## AOT 编译

AOT 编译器将 .sf/.sfl 项目编译为原生可执行文件：

```bash
# 使用 SF Runtime 进行 AOT 编译
sf build project.sf --target windows
sf build project.sf --target macos
sf build project.sf --target linux

# 编译流程：
# 1. 解析 .sf/.sfl 项目文件
# 2. 优化 IR
# 3. LLVM 19+ 生成目标平台机器码
# 4. 链接启动代码
# 5. 输出原生可执行文件
```

## 构建配置

### sf-core Cargo workspace 配置

`sf-core/Cargo.toml`：

```toml
[workspace]
members = [
    "sf-vm",
    "sf-blocks",
    "sf-renderer",
    "sf-parser",
    "sf-storage",
    "sf-audio",
]
resolver = "3"  # Cargo 2024 edition 使用 resolver 3
```

`sf-core/rust-toolchain.toml`：

```toml
[toolchain]
channel = "stable"
targets = ["wasm32-unknown-unknown"]
```

### sf-editor Next.js 配置

`sf-editor/next.config.ts`：

```typescript
import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  output: 'export',        // 静态导出
  turbopack: {
    rules: {
      '*.wasm': {
        loaders: ['wasm'],
      },
    },
  },
};

export default nextConfig;
```

## CI 构建

GitHub Actions 自动构建流程（`.github/workflows/pages.yml`）：

1. **Rust 构建**：`cargo build --release` + `wasm-pack build`
2. **前端构建**：`pnpm install` + `pnpm build`
3. **测试**：`cargo nextest run` + `pnpm test` + `pnpm e2e`
4. **部署**：Cloudflare Pages（Web 端）/ GitHub Releases（桌面端）
