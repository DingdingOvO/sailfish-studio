# 02 - 项目结构与模块职责

## 概述

Sailfish Studio 采用多仓库 + Monorepo 混合架构。核心 Rust crate 统一在 `sf-core` 工作区管理，独立功能各自建仓。本文档详细描述每个仓库的职责和依赖关系，确保开发者清楚模块边界和交互方式。

> 对应开发清单：`docs/development/09-implementation-checklist.md` Section 2
> 对应需求文档：`docs/requirements/03-architecture.md` Section 3.4

## 仓库总览

### 主仓库

| 仓库 | 包数 | 职责 |
|------|------|------|
| sf-core | 6 核心 crates（总计 28 包） | Rust 工作区：sf-vm, sf-blocks, sf-renderer, sf-parser, sf-storage, sf-audio |
| sf-editor | 13 包 | 编辑器 GUI (Next.js 16 + Wasm)，集成 sf-ui 框架、多窗口管理、插件引擎、本地化 |
| sf-tools | 8 包 | 打包器、解包器、CLI 工具、sb3fix、脚手架、后处理 |
| sf-services | 6 包 | 云端服务 (Cloudflare Workers)：云变量、账户、分享、统计、代理 |
| sf-extensions | 21+ 扩展 + 市场 | 扩展内容、市场网站、开发工具库 |
| sf-docs | — | 文档站 (VitePress) |
| sf-runtime | — | 独立运行时 (Rust CLI: sf run/pack/new/check) |
| sf-aot-compiler | — | AOT 编译器 (Rust + LLVM 19+) |

### sf-core 工作区 Crates 详解

sf-core 是 Rust Monorepo，使用 Cargo workspace 统一管理。以下是 6 个核心 crate：

| Crate | 职责 | 关键依赖 |
|-------|------|---------|
| sf-vm | 虚拟机核心：数据模型、编译器、执行器、扩展系统、设置引擎、WASM 导出 | wasm-bindgen, serde, thiserror, rusqlite |
| sf-blocks | 积木编辑器：Canvas 渲染、拖拽吸附、动态模糊拖尾、布局引擎 | wasm-bindgen, web-sys, lyon |
| sf-renderer | 舞台渲染器：WebGL2 渲染管线、SVG 解析、批量绘制 | wasm-bindgen, web-sys, resvg |
| sf-parser | 解析器：.sb3 (ZIP)、.sf (SQLite)、.sfl (文本) 格式解析 | serde, zip, rusqlite, thiserror |
| sf-storage | 存储引擎：项目文件读写、资源管理、增量保存 | rusqlite, serde |
| sf-audio | 音频引擎：音频播放、流式处理、缓存管理 | wasm-bindgen, web-sys |

### Fork 仓库（源自 TurboWarp）

Sailfish Studio 从 TurboWarp 分叉全部 44 个仓库，核心模块 Rust 重写后合并入 sf-core，其余保留并品牌化。

| 上游仓库 | SF 仓库 | 迁移方向 |
|----------|---------|----------|
| scratch-vm | sf-vm | JS 虚拟机 → Rust 引擎 (合并入 sf-core) |
| scratch-blocks | sf-blocks | Google Blockly → Canvas 自研 (合并入 sf-core) |
| scratch-render | sf-renderer | WebGL JS → Rust WebGL2 (合并入 sf-core) |
| scratch-gui | sf-editor | React → Next.js 16 + Rust UI 框架 |
| extensions | sf-extensions | 保留 + 扩展市场 |
| packager | sf-packager | 增强导出 (SWF/MP4/APK)，合并入 sf-tools |
| desktop | sf-desktop | Electron → Tauri 2.0，合并入 sf-editor |
| scratch-parser | sf-parser | JS → Rust (合并入 sf-core) |
| scratch-storage | sf-storage | JS → Rust (合并入 sf-core) |
| scratch-audio | sf-audio | JS → Rust (合并入 sf-core) |
| scratch-paint | sf-paint | 保留并品牌化 |
| scratch-l10n | sf-l10n | 保留并品牌化 |
| cloud-server | sf-cloud-server | 协作服务基础，合并入 sf-services |

### 新增仓库

| 仓库名 | 职责 |
|--------|------|
| sf-aot-compiler | AOT 编译器 (Rust + LLVM 19+)：将 .sf/.sfl 编译为原生可执行文件 |
| sf-runtime | 独立命令行运行时：`sf run`、`sf pack`、`sf new`、`sf check` |

## 关键模块依赖关系

```
sf-editor ──→ sf-core (通过 Wasm 导入)
sf-editor ──→ sf-tools (打包功能调用)
sf-extensions ──→ sf-core (扩展运行时依赖)
sf-desktop ──→ sf-editor (WebView 加载) + sf-core (sidecar 子进程)
sf-runtime ──→ sf-core (sf-vm 作为解释核心)
sf-aot-compiler ──→ sf-core (读取 .sf/.sfl 编译为原生代码)
sf-services ──↔ sf-core (云变量同步)
```

## 目录结构示例

### sf-core 目录结构

```
sf-core/
├── Cargo.toml              # Workspace 根配置
├── rust-toolchain.toml     # Rust 工具链配置
├── sf-vm/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # WASM 导出
│       ├── project/        # 数据模型与解析
│       ├── compiler/       # 编译器
│       ├── runtime/        # 执行器
│       ├── extension/      # 扩展系统
│       ├── settings/       # 设置引擎
│       └── ops/            # 积木操作函数
├── sf-blocks/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── layout.rs       # 积木布局引擎
│       ├── renderer.rs     # Canvas 渲染器
│       ├── drag.rs         # 拖拽与吸附
│       └── trail.rs        # 动态模糊拖尾
├── sf-renderer/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── webgl.rs        # WebGL2 渲染管线
│       ├── svg.rs          # SVG 解析与三角化
│       └── batch.rs        # 批量绘制
├── sf-parser/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── sb3.rs          # .sb3 解析
│       ├── sf.rs           # .sf 解析
│       └── sfl.rs          # .sfl 解析
├── sf-storage/
│   └── ...
└── sf-audio/
    └── ...
```

### sf-editor 目录结构

```
sf-editor/
├── package.json
├── next.config.ts
├── src/
│   ├── app/                # Next.js App Router 页面
│   ├── components/         # UI 组件库
│   ├── design/tokens/      # 设计令牌
│   ├── lib/
│   │   ├── wasm.ts         # WASM 模块加载
│   │   └── stores/         # Zustand 状态管理
│   └── i18n/               # 国际化
├── tests/
│   ├── unit/               # Vitest 单元测试
│   └── e2e/                # Playwright E2E 测试
└── public/
    └── wasm/               # 编译后的 WASM 文件
```
