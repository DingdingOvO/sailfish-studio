# Sailfish Studio 设计文档

**文档版本**：1.0-Beta.2
**应用版本**：1.0-Beta
**日期**：2026-06-07
**项目代号**：Sailfish Studio (SFS)

## 关于

本文档体系是 Sailfish Studio 的详细设计规范，覆盖进程架构、UI 设计语言、语言规范、协作协议、扩展系统、安全模型和测试策略。

## 阅读指南

按目录分类阅读，每个文件内包含完整的设计规范，可直接指导开发实施。

## 目录

### 架构设计
| 文件 | 内容 |
|------|------|
| [architecture/01-process-model.md](./architecture/01-process-model.md) | 进程模型与生命周期 |
| [architecture/02-ipc-protocol.md](./architecture/02-ipc-protocol.md) | 进程间通信协议 |
| [architecture/03-data-flow.md](./architecture/03-data-flow.md) | 数据流与状态管理 |
| [architecture/04-deployment.md](./architecture/04-deployment.md) | 部署拓扑 |

### UI 设计
| 文件 | 内容 |
|------|------|
| [ui/01-design-principles.md](./ui/01-design-principles.md) | 设计原则 |
| [ui/02-color-system.md](./ui/02-color-system.md) | 色彩体系 |
| [ui/03-typography.md](./ui/03-typography.md) | 字体与排版 |
| [ui/04-spacing-grid.md](./ui/04-spacing-grid.md) | 间距与网格 |
| [ui/05-iconography.md](./ui/05-iconography.md) | 图标系统 |
| [ui/06-motion.md](./ui/06-motion.md) | 动画规范 |
| [ui/08-layout.md](./ui/08-layout.md) | 编辑器布局 |
| [ui/09-dark-mode.md](./ui/09-dark-mode.md) | 深色模式 |
| [ui/10-accessibility.md](./ui/10-accessibility.md) | 无障碍设计 |
| [ui/11-mobile-tablet.md](./ui/11-mobile-tablet.md) | 平板/触控适配 |

#### 组件规范
| 文件 | 内容 |
|------|------|
| [ui/07-components/block-canvas.md](./ui/07-components/block-canvas.md) | 积木画布 |
| [ui/07-components/button.md](./ui/07-components/button.md) | 按钮 |
| [ui/07-components/dialog.md](./ui/07-components/dialog.md) | 对话框 |
| [ui/07-components/dropdown.md](./ui/07-components/dropdown.md) | 下拉菜单 |
| [ui/07-components/input.md](./ui/07-components/input.md) | 输入框 |
| [ui/07-components/panel.md](./ui/07-components/panel.md) | 面板 |
| [ui/07-components/stage-canvas.md](./ui/07-components/stage-canvas.md) | 舞台画布 |
| [ui/07-components/status-bar.md](./ui/07-components/status-bar.md) | 状态栏 |
| [ui/07-components/tab-bar.md](./ui/07-components/tab-bar.md) | 选项卡 |
| [ui/07-components/toolbar.md](./ui/07-components/toolbar.md) | 工具栏 |
| [ui/07-components/tree-view.md](./ui/07-components/tree-view.md) | 树形视图 |

### 语言设计
| 文件 | 内容 |
|------|------|
| [language/01-lexical-grammar.md](./language/01-lexical-grammar.md) | SF 语言词法规则 |
| [language/02-syntax.md](./language/02-syntax.md) | SF 语言语法规范 |
| [language/03-type-system.md](./language/03-type-system.md) | 类型系统设计 |
| [language/04-block-mapping.md](./language/04-block-mapping.md) | 积木 ↔ .sfl 映射规则 |
| [language/05-standard-library.md](./language/05-standard-library.md) | SF 标准库 |
| [language/06-vscode-extension.md](./language/06-vscode-extension.md) | VS Code 扩展设计 |

### 扩展系统
| 文件 | 内容 |
|------|------|
| [extensions/01-extension-api.md](./extensions/01-extension-api.md) | 扩展 API 参考 |
| [extensions/02-extension-lifecycle.md](./extensions/02-extension-lifecycle.md) | 扩展生命周期 |
| [extensions/03-plugin-api.md](./extensions/03-plugin-api.md) | 插件 API 参考 |
| [extensions/04-sandbox-model.md](./extensions/04-sandbox-model.md) | 沙箱模型 |
| [extensions/05-marketplace.md](./extensions/05-marketplace.md) | 扩展与插件市场 |

### 协作设计
| 文件 | 内容 |
|------|------|
| [collaboration/01-protocol.md](./collaboration/01-protocol.md) | 协作协议 |
| [collaboration/02-room-server.md](./collaboration/02-room-server.md) | 房间服务器架构 |
| [collaboration/03-presence.md](./collaboration/03-presence.md) | 用户存在感设计 |
| [collaboration/04-conflict-resolution.md](./collaboration/04-conflict-resolution.md) | 冲突解决算法 |
| [collaboration/05-offline-sync.md](./collaboration/05-offline-sync.md) | 离线同步设计 |

### 安全
| 文件 | 内容 |
|------|------|
| [security/01-threat-model.md](./security/01-threat-model.md) | 威胁模型 |
| [security/02-sandbox-boundaries.md](./security/02-sandbox-boundaries.md) | 沙箱边界 |
| [security/03-permission-model.md](./security/03-permission-model.md) | 权限模型 |
| [security/04-csp-policy.md](./security/04-csp-policy.md) | CSP 策略 |
| [security/05-code-signing.md](./security/05-code-signing.md) | 代码签名 |

### 测试
| 文件 | 内容 |
|------|------|
| [testing/01-test-pyramid.md](./testing/01-test-pyramid.md) | 测试金字塔 |
| [testing/02-performance-bench.md](./testing/02-performance-bench.md) | 性能基准 |
| [testing/03-compatibility-matrix.md](./testing/03-compatibility-matrix.md) | 兼容性矩阵 |
| [testing/04-fuzzing-strategy.md](./testing/04-fuzzing-strategy.md) | 模糊测试策略 |
