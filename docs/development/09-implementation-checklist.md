# 09 - 实际开发清单

文档版本：1.0-Beta.1
对应需求：docs/requirements/
对应设计：docs/design/
状态：施工中

---

## 清单结构

```
docs/development/
├── 09-implementation-checklist.md    # 本文件：总览与阶段说明
├── 09a-phase1-infrastructure.md      # Phase 1：组织与基础设施
├── 09b-phase2-core-engine.md         # Phase 2：核心引擎 Rust 重写
├── 09c-phase3-editor-ui.md           # Phase 3：编辑器 UI 框架
├── 09d-phase4-collaboration.md       # Phase 4：多人协作系统
├── 09e-phase5-runtime-packager.md    # Phase 5：运行时与打包器
├── 09f-phase6-desktop-mobile.md      # Phase 6：桌面端与移动端
├── 09g-phase7-extensions-market.md   # Phase 7：扩展市场与生态
├── 09h-phase8-testing-publish.md     # Phase 8：测试、文档与发布
```

---

## 总览

Sailfish Studio 开发工作分为 8 个阶段，按依赖关系顺序推进。本清单基于从 TurboWarp JS 架构到 Rust 独立平台的完整迁移蓝图。

| 阶段 | 名称 | 预计周期 | 关键产出 |
|------|------|----------|----------|
| Phase 1 | 组织与基础设施 | 2 周 | GitHub 组织、仓库、CI/CD |
| Phase 2 | 核心引擎 | 12 周 | sf-vm、sf-blocks、sf-renderer、sf-parser |
| Phase 3 | 编辑器 UI | 10 周 | sf-editor、sf-ui 框架 |
| Phase 4 | 多人协作 | 8 周 | 协作服务器、OT 协议、用户系统 |
| Phase 5 | 运行时与打包器 | 6 周 | SF Runtime、sf-packager、AOT 编译器 |
| Phase 6 | 桌面端与移动端 | 6 周 | Tauri 桌面端、Android 平板端 |
| Phase 7 | 扩展市场与生态 | 4 周 | sf-extensions 市场、高级扩展 |
| Phase 8 | 测试、文档与发布 | 4 周 | 全量测试、用户文档、v1.0-Beta |

---

## 阶段依赖关系

```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4
                │            │
                └──→ Phase 5 ──→ Phase 6
                                    │
Phase 7 ────────────────────────────┤
                                    │
Phase 8 ────────────────────────────┘
```

- Phase 2 和 Phase 3 是核心路径，必须顺序执行
- Phase 5 依赖 Phase 2，可与 Phase 3 部分并行
- Phase 6 依赖 Phase 3 和 Phase 5
- Phase 7 可与 Phase 4-6 并行
- Phase 8 在所有模块完成后执行

---

## 子清单导航

| 子清单 | 内容 | 步骤数 |
|--------|------|--------|
| 09a-phase1-infrastructure.md | 组织创建、仓库分叉、CI/CD 配置、域名与邮箱 | ~50 |
| 09b-phase2-core-engine.md | sf-vm 数据模型、编译器、执行器、sf-blocks 积木引擎、sf-renderer 渲染器 | ~150 |
| 09c-phase3-editor-ui.md | sf-ui 框架、编辑器布局、设计语言、多语言、可访问性 | ~100 |
| 09d-phase4-collaboration.md | 协作协议、房间服务器、用户系统、离线同步 | ~80 |
| 09e-phase5-runtime-packager.md | SF Runtime CLI、AOT 编译器、sf-packager 多格式导出 | ~60 |
| 09f-phase6-desktop-mobile.md | Tauri 集成、原生窗口、平板 UI 适配 | ~50 |
| 09g-phase7-extensions-market.md | 扩展市场网站、首批高级扩展开发 | ~40 |
| 09h-phase8-testing-publish.md | 全量测试、文档站、发布流程 | ~50 |

总计约 580 个施工步骤。

---

## 一、组织与仓库层（GitHub）

### 1.1 创建组织

- [ ] 创建 GitHub 组织：Sailfish-Studio
- [ ] 配置组织团队：core（核心开发）、docs（文档）、triage（问题分流）
- [ ] 启用组织安全策略：双因素认证、SAML SSO（如需）
- [ ] 配置组织级别的 Dependabot 和 secret scanning

### 1.2 Fork 上游仓库并重命名

将 TurboWarp 下所有仓库 Fork 到组织，并重命名：

| 上游仓库 | 新名称 | 用途 |
|----------|--------|------|
| scratch-vm | sf-vm | 虚拟机 → Rust 引擎 |
| scratch-blocks | sf-blocks | 积木引擎 → Canvas 自研 |
| scratch-render | sf-renderer | 渲染器 → Rust WebGL2 |
| scratch-gui | sf-editor | 编辑器 → Rust UI 框架 |
| extensions | sf-extensions | 扩展市场 + 高级扩展 |
| packager | sf-packager | 打包器增强 |
| desktop | sf-desktop | Electron → Tauri |
| scratch-parser | sf-parser | 解析器 → Rust |
| 其他辅助库 | sf-* 前缀 | 全部带上 sf- 前缀 |

### 1.3 新增仓库（Monorepo 体系）

| 新仓库名 | 用途 |
|----------|------|
| sf-core | Rust 工作区，合并所有核心 Rust crate |
| sf-tools | 合并打包器、CLI 等工具 |
| sf-services | 云端服务（Cloudflare Workers） |
| sf-docs | 文档站（VitePress） |
| sf-runtime | 独立运行时（命令行工具） |
| sf-aot-compiler | AOT 编译器（Rust + LLVM） |

---

## 二、代码层：每个仓库的具体修改动作

### 2.1 sf-vm（原 scratch-vm）

**目标：JS 虚拟机 → Rust 引擎**

- [ ] 删除所有 JavaScript 源码（保留旧版本在 Git 历史）
- [ ] 添加 `Cargo.toml`，配置 `wasm-bindgen`, `serde`, `zip` 等依赖
- [ ] 新建 `src/` 下的 Rust 模块：
  - [ ] `src/project/`：加载 .sf (SQLite), .sb3, .sfl 解析
  - [ ] `src/compiler/`：积木 AST → JavaScript 代码生成
  - [ ] `src/runtime/`：状态管理、操作执行器（运动、外观等）
  - [ ] `src/extension/`：扩展系统接口（保留 JS 扩展兼容）
  - [ ] `src/settings/`：分层设置引擎
- [ ] 保留但修改 `Scratch.extensions.register` 接口，内部改由 Rust 实现桥接
- [ ] 新增编译为 Wasm 的入口 `lib.rs`，导出 `sf_vm_create` 等 API（对应 `docs/api/01-sf-vm-api.md`）

### 2.2 sf-blocks（原 scratch-blocks）

**目标：Google Blockly → 自研 Canvas 引擎**

- [ ] 删除所有 Blockly 相关 JS 代码
- [ ] 添加 `Cargo.toml`，依赖 `web-sys` (Canvas 2D)
- [ ] 新建 `src/` 下模块：
  - [ ] `src/layout/`：积木几何布局
  - [ ] `src/renderer/`：Canvas 绘制
  - [ ] `src/drag/`：拖拽与吸附（对应 `docs/design/ui/07-components/block-canvas.md`）
  - [ ] `src/trail/`：动态模糊拖尾
  - [ ] `src/toolbox/`：工具箱
  - [ ] `src/history/`：撤销/重做
- [ ] 实现与 sf-vm 通过 JSON 交互的序列化接口

### 2.3 sf-renderer（原 scratch-render）

**目标：WebGL JS → Rust + WebGL2**

- [ ] 删除原始 JS 代码
- [ ] 添加 `Cargo.toml`，依赖 `web-sys` (WebGL2), `resvg`, `lyon`
- [ ] 新建 `src/` 下模块：
  - [ ] `src/webgl/`：着色器、缓冲区、纹理
  - [ ] `src/svg/`：SVG 解析与三角化
  - [ ] `src/commands/`：绘制指令队列

### 2.4 sf-editor（原 scratch-gui）

**目标：React 界面 → Rust 自研 UI 框架**

- [ ] 删除所有 React 组件（`src/components/`, `src/containers/` 等）
- [ ] 保留并适配 `src/addons/` 中的社区插件接口
- [ ] 添加 `Cargo.toml` 作为 Rust 项目入口（通过 `wasm-pack` 构建）
- [ ] 新建 Rust UI 框架（sf-ui），实现：
  - [ ] 设计令牌系统（颜色、间距、字体）— 对应 `docs/design/ui/01-design-principles.md` 至 `05-iconography.md`
  - [ ] 基础组件库（按钮、输入框、弹窗、树等）— 对应 `docs/design/ui/07-components/`
  - [ ] 编辑器布局（标题栏、菜单、工具栏、面板）— 对应 `docs/design/ui/08-layout.md`
- [ ] 集成 sf-blocks 和 sf-renderer 的 Canvas
- [ ] 配置 Turbopack 构建配置（`next.config.ts`），打包所有 Wasm 模块

### 2.5 sf-extensions（原 extensions）

**目标：扩展市场 + 高级扩展合集**

- [ ] 保留原有 `extensions/` 目录中的所有 JS 扩展
- [ ] 为每个扩展添加 `extension.json`
- [ ] 新建 21 个高级扩展的目录和代码（Rust/C++/TS）
- [ ] 修改市场网站前端（可用 Rust 重写或保留原样但品牌化）

### 2.6 sf-packager（原 packager）

**目标：打包器增强**

- [ ] 保留 HTML/ZIP 打包核心逻辑
- [ ] 新增 SWF 导出模块（集成 Ruffle）
- [ ] 新增 MP4/GIF 导出模块（WebCodecs）
- [ ] 新增 APK 导出模块（Tauri 移动端）
- [ ] 修改品牌化，内部常量替换为 SF

### 2.7 sf-desktop（原 desktop）

**目标：Electron → Tauri**

- [ ] 删除 `node_modules/`, Electron 相关配置
- [ ] 添加 `src-tauri/` 及 `Cargo.toml`
- [ ] 配置 `tauri.conf.json`：无边框窗口，指定 sf-editor 构建产物为前端
- [ ] 实现 Rust 侧 Tauri 命令：文件对话框、自动更新、SQLite 存储

### 2.8 新增仓库：sf-runtime

- [ ] 新建独立的 Rust 项目，使用 `clap` 构建 CLI
- [ ] 实现 `sf run`：运行 .sf/.sfl 项目
- [ ] 实现 `sf pack`：打包项目
- [ ] 实现 `sf new`：新建项目
- [ ] 实现 `sf check`：检查项目
- [ ] 集成 sf-vm 作为解释核心

### 2.9 新增仓库：sf-aot-compiler

- [ ] 新建 Rust 项目，后端使用 LLVM（`cranelift` 或 `llvm-sys`）
- [ ] 实现读取 .sf/.sfl，输出原生可执行文件
- [ ] 支持目标平台：Windows / macOS / Linux

---

## 三、构建系统与代码规范修改

### 3.1 Monorepo 配置

- [ ] 在根目录建立 `Cargo.toml` (workspace)
- [ ] 在根目录建立 `pnpm-workspace.yaml`
- [ ] 配置 workspace 成员路径

### 3.2 Turbopack 配置

- [ ] 在 `sf-editor/next.config.ts` 中添加 WASM 加载规则
- [ ] 配置 `wasm-pack` 构建产物输出目录
- [ ] 配置 Turbopack dev server 与 `wasm-pack` 的热重载联动

### 3.3 代码规范强制

- [ ] 禁止魔法数字：所有数值必须定义为常量（如 `MAGNETIC_SNAP_DISTANCE_PX = 8`）
- [ ] Rust Lint：`cargo clippy -- -D warnings`
- [ ] JS/TS Lint：Biome 严格模式
- [ ] 格式化：`cargo fmt` + Biome format
- [ ] 所有公共 API 必须有文档注释（`///` / JSDoc）

### 3.4 测试配置

- [ ] Rust 单元测试：`cargo-nextest` + `wasm-bindgen-test`
- [ ] 前端测试：Vitest + Playwright
- [ ] CLI 集成测试：pytest (Python)
- [ ] 模糊测试：`cargo-fuzz`
- [ ] 覆盖率：`cargo-tarpaulin` (Rust) / Vitest coverage (TS)

---

## 四、文档体系建立

在 `docs/` 下按已认可的结构放置所有文档的 .md 文件，保持多文件引用。

| 文档集 | 目录 | 文件数 | 状态 |
|--------|------|--------|------|
| 需求文档 | `docs/requirements/` | 15 | ✅ 已完成 |
| 设计文档 | `docs/design/` | 34 | ✅ 已完成 |
| 开发文档 | `docs/development/` | 9 | ✅ 已完成（含本文件） |
| API 参考 | `docs/api/` | 5 | ✅ 已完成 |
| 运维文档 | `docs/operations/` | 5 | ✅ 已完成 |
| 用户文档 | `docs/user/` | 6 | ✅ 已完成 |

---

## 阅读指南

- 每份子清单包含具体的、可检查的任务项
- 每项任务对应到具体的仓库和模块
- 任务完成后在 `[ ]` 中标记为 `[x]`
- 任务阻塞项（前置依赖）在任务描述中注明
