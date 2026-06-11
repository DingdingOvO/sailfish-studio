# 02 - 项目结构与模块职责

## 概述

Sailfish Studio 采用 Monorepo 架构，6 个主仓库，内部通过 workspace 管理 80+ 细粒度包。

## 仓库总览

| 仓库 | 包数 | 职责 |
|------|------|------|
| sf-core | 28 | 核心运行时、引擎、语言工具链 |
| sf-editor | 13 | 编辑器 GUI、UI 框架、插件、社区 |
| sf-tools | 8 | 打包器、CLI、工具 |
| sf-services | 6 | 云端服务 |
| sf-extensions | 21+ | 扩展市场与内容 |
| sf-docs | — | 文档站 |

## 关键模块依赖关系

- sf-editor → sf-core (通过 Wasm 导入)
- sf-editor → sf-tools (打包功能调用)
- sf-extensions → sf-core (扩展运行时依赖)
- 桌面端 → sf-editor (WebView 加载) + sf-core (sidecar 子进程)
