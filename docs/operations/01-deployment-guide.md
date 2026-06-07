# 01 - 部署指南

## Web 端 (Cloudflare Pages)

1. 在 Cloudflare Dashboard 创建 Pages 项目
2. 关联 sf-editor 仓库
3. 构建命令: `pnpm build`
4. 构建输出目录: `dist/`
5. 绑定域名: editor.sailfish.studio

## 扩展市场 (Cloudflare Pages)

1. 关联 sf-extensions 仓库
2. 构建命令: `pnpm build`
3. 构建输出目录: `dist/`
4. 绑定域名: extensions.sailfish.studio

## 文档站 (Cloudflare Pages)

1. 关联 sf-docs 仓库
2. 构建命令: `pnpm build` (VitePress)
3. 绑定域名: docs.sailfish.studio

## 云变量服务 (Cloudflare Workers)

1. 进入 sf-services 目录
2. 运行 `wrangler deploy`
3. 绑定路由: api.sailfish.studio/*

## 桌面端 (GitHub Releases)

1. 运行 `cargo tauri build`
2. 上传构建产物到 GitHub Releases
3. 生成更新清单 (update.json)
