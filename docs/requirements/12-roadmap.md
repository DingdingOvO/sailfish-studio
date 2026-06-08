# 12 - 路线图

[上一章](./11-nfr.md) | [下一章：附录](./13-appendices.md)

| Phase | 主题 | 主要交付物 |
|-------|------|-----------|
| 1 | 组织与仓库搭建 | GitHub 组织、Fork 仓库、Monorepo 配置、CI/CD 流水线 |
| 2 | 核心引擎 Rust 重写 | sf-vm、sf-blocks、sf-renderer、sf-parser、WASM 导出 |
| 3 | 编辑器 UI 框架 | sf-editor (Next.js 16)、设计令牌、UI 组件库、编辑器布局、WASM 集成 |
| 4 | 多人协作系统 | 协作服务器、OT 算法、用户存在感、用户系统、离线同步 |
| 5 | 运行时与打包器 | SF Runtime CLI、sf-packager 增强、AOT 编译器 (LLVM 19+) |
| 6 | 桌面端与移动端 | Tauri 2.0 桌面端、Android 平板端、代码签名 |
| 7 | 扩展市场与生态 | 扩展市场网站、21 个首批高级扩展、开发者文档 |
| 8 | 测试与发布 | 完整测试套件、性能基准验证、安全审计、发布 v1.0-Beta |

> 详细开发步骤见 `docs/development/09-implementation-checklist.md`
