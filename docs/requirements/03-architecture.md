# 03 - 技术架构

[上一章](./02-concepts.md) | [下一章：积木编辑器](./04-editor.md)

## 3.1 全栈技术栈
- 核心语言：Rust 1.87+（edition 2024），编译为 wasm32-unknown-unknown
- 前端构建：Next.js 16.2 (Turbopack)
- 桌面端：Tauri 2.0
- 包管理：pnpm 10+
- 数据库：SQLite
- C/C++ 扩展：Emscripten 4.2
- AOT 后端：LLVM 19+

## 3.2 多进程架构

Sailfish Studio 采用三进程架构确保稳定性。

| 进程 | 职责 | 生命周期 | 数量 |
|------|------|---------|------|
| 主进程 | 窗口管理、菜单、系统托盘、自动更新、IPC 中枢 | 启动到退出 | 1 |
| 渲染进程 | 编辑器 UI（积木区、舞台、面板） | 启动到退出 | 1（常驻） |
| 扩展进程 | 项目执行（VM、编译、扩展） | 按项目 | 每项目 1 个 |

**关键约束**：
- 渲染进程只负责 UI，不执行项目逻辑。
- 每个项目运行在独立扩展进程中，完全隔离。
- 打开新项目不重新加载渲染进程。
- 关闭项目则终止扩展进程，内存立即释放。

**浏览器端实现**：渲染进程 = 主线程，扩展进程 = Web Worker。
**桌面端实现**：渲染进程 = WebView，扩展进程 = Tauri sidecar 子进程（推荐）或 Web Worker（降级）。

## 3.3 源代码质量要求
- Rust：`cargo fmt`、`cargo clippy -D warnings`、零 `unsafe`（除非审查批准）、禁止 `unwrap()` 在生产代码中、测试覆盖率 ≥ 95%。
- TS/JS：Biome 格式化和 lint、禁止 `any`（除非审查批准）、禁止魔法数字、测试覆盖率 ≥ 95%。
- 代码审查：所有 PR 必须审查通过，CI 必须全绿。

## 3.4 仓库组织（8 个主仓库）
- **sf-core** (6 核心 crates，总计 28 包)：VM、编译器、积木引擎、渲染器、解析器、存储、音频
- **sf-editor** (13 包)：UI 框架、编辑器主应用、多窗口管理、addons、插件引擎、插件市场、社区、平板 UI、调试控制台、本地化
- **sf-tools** (8 包)：打包器、解包器、CLI、sb3fix、脚手架、后处理
- **sf-services** (6 包)：云变量、账户、分享、统计、代理、轻量 API
- **sf-extensions** (21+ 扩展 + 市场)：扩展内容、市场网站、开发工具库
- **sf-docs**：文档站 (VitePress)
- **sf-runtime**：独立命令行运行时 (Rust CLI: sf run/pack/new/check)
- **sf-aot-compiler**：AOT 编译器 (Rust + LLVM)
