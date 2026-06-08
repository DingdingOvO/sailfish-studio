# Sailfish Studio 文档索引

> 完整文档路径索引，可通过 `?path=<路径>` 渲染查看，或 `?path=<路径>&raw` 查看纯文本源码。

## 需求规格

| 编号 | 文档 | 路径 |
|------|------|------|
| 00 | 文档约定与常量 | `docs/requirements/00-constants.md` |
| 01 | 项目概述 | `docs/requirements/01-overview.md` |
| 02 | 核心概念 | `docs/requirements/02-concepts.md` |
| 03 | 技术架构 | `docs/requirements/03-architecture.md` |
| 04 | 积木编辑器 | `docs/requirements/04-editor.md` |
| 05 | 多人协作 | `docs/requirements/05-collaboration.md` |
| 06 | UI 与设计语言 | `docs/requirements/06-ui.md` |
| 07 | 扩展与插件 | `docs/requirements/07-extensions.md` |
| 08 | 运行时与打包 | `docs/requirements/08-runtime.md` |
| 09 | 跨平台 | `docs/requirements/09-platform.md` |
| 10 | 调试与日志 | `docs/requirements/10-debug.md` |
| 11 | 非功能性需求 | `docs/requirements/11-nfr.md` |
| 12 | 路线图 | `docs/requirements/12-roadmap.md` |
| 13 | 附录 | `docs/requirements/13-appendices.md` |
| 14 | 变更历史 | `docs/requirements/14-changelog.md` |

## 架构设计

| 编号 | 文档 | 路径 |
|------|------|------|
| A1 | 进程模型 | `docs/design/architecture/01-process-model.md` |
| A2 | IPC 协议 | `docs/design/architecture/02-ipc-protocol.md` |
| A3 | 数据流 | `docs/design/architecture/03-data-flow.md` |
| A4 | 部署拓扑 | `docs/design/architecture/04-deployment.md` |

## UI 设计 - 基础

| 编号 | 文档 | 路径 |
|------|------|------|
| U1 | 设计原则 | `docs/design/ui/01-design-principles.md` |
| U2 | 色彩体系 | `docs/design/ui/02-color-system.md` |
| U3 | 字体排版 | `docs/design/ui/03-typography.md` |
| U4 | 间距网格 | `docs/design/ui/04-spacing-grid.md` |
| U5 | 图标系统 | `docs/design/ui/05-iconography.md` |
| U6 | 动画规范 | `docs/design/ui/06-motion.md` |

## UI 设计 - 组件

| 编号 | 文档 | 路径 |
|------|------|------|
| C1 | 按钮 | `docs/design/ui/07-components/button.md` |
| C2 | 输入框 | `docs/design/ui/07-components/input.md` |
| C3 | 对话框 | `docs/design/ui/07-components/dialog.md` |
| C4 | 下拉菜单 | `docs/design/ui/07-components/dropdown.md` |
| C5 | 选项卡 | `docs/design/ui/07-components/tab-bar.md` |
| C6 | 工具栏 | `docs/design/ui/07-components/toolbar.md` |
| C7 | 面板 | `docs/design/ui/07-components/panel.md` |
| C8 | 树形视图 | `docs/design/ui/07-components/tree-view.md` |
| C9 | 积木画布 | `docs/design/ui/07-components/block-canvas.md` |
| CA | 舞台画布 | `docs/design/ui/07-components/stage-canvas.md` |
| CB | 状态栏 | `docs/design/ui/07-components/status-bar.md` |

## UI 设计 - 布局与适配

| 编号 | 文档 | 路径 |
|------|------|------|
| U7 | 编辑器布局 | `docs/design/ui/08-layout.md` |
| U8 | 深色模式 | `docs/design/ui/09-dark-mode.md` |
| U9 | 无障碍 | `docs/design/ui/10-accessibility.md` |
| UA | 平板适配 | `docs/design/ui/11-mobile-tablet.md` |

## 语言设计

| 编号 | 文档 | 路径 |
|------|------|------|
| L1 | 词法规则 | `docs/design/language/01-lexical-grammar.md` |
| L2 | 语法规范 | `docs/design/language/02-syntax.md` |
| L3 | 类型系统 | `docs/design/language/03-type-system.md` |
| L4 | 积木映射 | `docs/design/language/04-block-mapping.md` |
| L5 | 标准库 | `docs/design/language/05-standard-library.md` |
| L6 | VS Code 扩展 | `docs/design/language/06-vscode-extension.md` |

## 扩展系统

| 编号 | 文档 | 路径 |
|------|------|------|
| E1 | 扩展 API | `docs/design/extensions/01-extension-api.md` |
| E2 | 扩展生命周期 | `docs/design/extensions/02-extension-lifecycle.md` |
| E3 | 插件 API | `docs/design/extensions/03-plugin-api.md` |
| E4 | 沙箱模型 | `docs/design/extensions/04-sandbox-model.md` |
| E5 | 扩展市场 | `docs/design/extensions/05-marketplace.md` |

## 协作设计

| 编号 | 文档 | 路径 |
|------|------|------|
| CO1 | 协作协议 | `docs/design/collaboration/01-protocol.md` |
| CO2 | 房间服务器 | `docs/design/collaboration/02-room-server.md` |
| CO3 | 用户存在感 | `docs/design/collaboration/03-presence.md` |
| CO4 | 冲突解决 | `docs/design/collaboration/04-conflict-resolution.md` |
| CO5 | 离线同步 | `docs/design/collaboration/05-offline-sync.md` |

## 安全

| 编号 | 文档 | 路径 |
|------|------|------|
| S1 | 威胁模型 | `docs/design/security/01-threat-model.md` |
| S2 | 沙箱边界 | `docs/design/security/02-sandbox-boundaries.md` |
| S3 | 权限模型 | `docs/design/security/03-permission-model.md` |
| S4 | CSP 策略 | `docs/design/security/04-csp-policy.md` |
| S5 | 代码签名 | `docs/design/security/05-code-signing.md` |

## 测试

| 编号 | 文档 | 路径 |
|------|------|------|
| T1 | 测试金字塔 | `docs/design/testing/01-test-pyramid.md` |
| T2 | 性能基准 | `docs/design/testing/02-performance-bench.md` |
| T3 | 兼容性矩阵 | `docs/design/testing/03-compatibility-matrix.md` |
| T4 | 模糊测试 | `docs/design/testing/04-fuzzing-strategy.md` |

## 开发文档

| 编号 | 文档 | 路径 |
|------|------|------|
| D1 | 开发环境搭建 | `docs/development/01-environment-setup.md` |
| D2 | 项目结构 | `docs/development/02-project-structure.md` |
| D3 | 编码规范 | `docs/development/03-coding-standards.md` |
| D4 | Git 工作流 | `docs/development/04-git-workflow.md` |
| D5 | 构建系统 | `docs/development/05-build-system.md` |
| D6 | 测试指南 | `docs/development/06-testing-guide.md` |
| D7 | 调试指南 | `docs/development/07-debugging-guide.md` |
| D8 | 发布流程 | `docs/development/08-release-process.md` |
| D9 | 实际开发清单 | `docs/development/09-implementation-checklist.md` |

## API 参考

| 编号 | 文档 | 路径 |
|------|------|------|
| R1 | sf-vm WASM 接口 | `docs/api/01-sf-vm-api.md` |
| R2 | 扩展 API | `docs/api/02-extension-api.md` |
| R3 | 插件 API | `docs/api/03-plugin-api.md` |
| R4 | IPC 协议 | `docs/api/04-ipc-api.md` |
| R5 | 协作 API | `docs/api/05-collab-api.md` |

## 运维文档

| 编号 | 文档 | 路径 |
|------|------|------|
| O1 | 部署指南 | `docs/operations/01-deployment-guide.md` |
| O2 | 监控与告警 | `docs/operations/02-monitoring.md` |
| O3 | 备份与恢复 | `docs/operations/03-backup-restore.md` |
| O4 | 事件响应 | `docs/operations/04-incident-response.md` |
| O5 | 性能调优 | `docs/operations/05-performance-tuning.md` |

## 用户文档

| 编号 | 文档 | 路径 |
|------|------|------|
| U1 | 快速入门 | `docs/user/01-getting-started.md` |
| U2 | 积木参考 | `docs/user/02-block-reference.md` |
| U3 | .sfl 语言指南 | `docs/user/03-sfl-guide.md` |
| U4 | 扩展使用指南 | `docs/user/04-extension-guide.md` |
| U5 | 协作指南 | `docs/user/05-collaboration-guide.md` |
| U6 | 常见问题 | `docs/user/06-faq.md` |
