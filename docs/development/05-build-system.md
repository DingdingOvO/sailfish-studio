# 05 - 构建系统

## 概述

使用 Turbopack 作为统一构建工具。构建流程: wasm-pack build → Turbopack 打包。

## 构建命令

```bash
# Web 端
cd sf-editor
pnpm build          # 生产构建
pnpm dev            # 开发服务器

# 桌面端
cd sf-desktop
cargo tauri build   # 原生应用打包

# AOT 编译
sf build project.sf --target windows
```

## 构建产物

- Web 端: `sf-editor/dist/` (静态站点)
- 桌面端: `.dmg` / `.msi` / `.AppImage`
- 运行时: 原生二进制文件
