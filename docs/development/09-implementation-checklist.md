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

Sailfish Studio 开发工作分为 8 个阶段，按依赖关系顺序推进。

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

## 阅读指南

- 每份子清单包含具体的、可检查的任务项
- 每项任务对应到具体的仓库和模块
- 任务完成后在 `[ ]` 中标记为 `[x]`
- 任务阻塞项（前置依赖）在任务描述中注明
