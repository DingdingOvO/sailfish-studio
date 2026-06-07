# 04 Deployment

04 - 部署拓扑

1. 概述

Sailfish Studio 支持两种部署模式：静态 Web 站点和桌面应用。

2. Web 端部署 (Cloudflare Pages)

· 构建产物: sf-editor 的 Next.js 静态导出 (output: 'export')
· 部署到 Cloudflare Pages，绑定域名 editor.sailfish.studio
· 扩展市场部署到 extensions.sailfish.studio
· 文档站部署到 docs.sailfish.studio
· 云变量 Worker 部署到 api.sailfish.studio

3. 桌面端部署 (GitHub Releases)

· 构建产物: Tauri 打包的 .dmg (macOS)、.msi (Windows)、.AppImage (Linux)
· 通过 GitHub Releases 分发，集成自动更新
· 代码签名: macOS 需 Developer ID 公证，Windows 需 EV Code Signing Certificate

4. 扩展进程部署

· 桌面端: 扩展进程二进制包含在 Tauri 安装包中，位于 resources/sidecars/
· Web 端: 扩展进程 Wasm 模块部署到 CDN，按需下载
