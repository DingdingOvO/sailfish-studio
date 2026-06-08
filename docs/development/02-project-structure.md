# 02 - 项目结构与模块职责

## 概述

Sailfish Studio 采用多仓库 + Monorepo 混合架构。核心 Rust crate 统一在 `sf-core` 工作区管理，独立功能各自建仓。

> 对应开发清单：`docs/development/09-implementation-checklist.md` Section 2

## 仓库总览

### 主仓库

| 仓库 | 包数 | 职责 |
|------|------|------|
| sf-core | 6+ crates | Rust 工作区：sf-vm, sf-blocks, sf-renderer, sf-parser, sf-storage, sf-audio |
| sf-editor | — | 编辑器 GUI (Next.js + Wasm)，集成 sf-ui 框架 |
| sf-tools | — | 打包器、CLI 工具 |
| sf-services | — | 云端服务 (Cloudflare Workers) |
| sf-extensions | 21+ | 扩展市场与内容 |
| sf-docs | — | 文档站 (VitePress) |
| sf-runtime | — | 独立运行时 (Rust CLI: sf run/pack/new/check) |

### Fork 仓库（源自 TurboWarp）

| 原始仓库 | Fork 名称 | 迁移方向 |
|----------|-----------|----------|
| scratch-vm | sf-vm | JS 虚拟机 → Rust 引擎 (合并入 sf-core) |
| scratch-blocks | sf-blocks | Google Blockly → Canvas 自研 (合并入 sf-core) |
| scratch-render | sf-renderer | WebGL JS → Rust WebGL2 (合并入 sf-core) |
| scratch-gui | sf-editor | React → Rust UI 框架 |
| extensions | sf-extensions | 保留 + 扩展市场 |
| packager | sf-packager | 增强导出 (SWF/MP4/APK) |
| desktop | sf-desktop | Electron → Tauri |
| scratch-parser | sf-parser | JS → Rust (合并入 sf-core) |
| scratch-storage | sf-storage | JS → Rust (合并入 sf-core) |
| scratch-audio | sf-audio | JS → Rust (合并入 sf-core) |
| scratch-paint | sf-paint | 保留并品牌化 |
| scratch-l10n | sf-l10n | 保留并品牌化 |
| cloud-server | sf-cloud-server | 协作服务基础 |

### 新增仓库

| 仓库名 | 职责 |
|--------|------|
| sf-aot-compiler | AOT 编译器 (Rust + LLVM) |

## 关键模块依赖关系

- sf-editor → sf-core (通过 Wasm 导入)
- sf-editor → sf-tools (打包功能调用)
- sf-extensions → sf-core (扩展运行时依赖)
- sf-desktop → sf-editor (WebView 加载) + sf-core (sidecar 子进程)
- sf-runtime → sf-core (sf-vm 作为解释核心)
- sf-aot-compiler → sf-core (读取 .sf/.sfl 编译为原生代码)
