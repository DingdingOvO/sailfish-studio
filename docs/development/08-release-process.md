# 08 - 发布流程

## 版本号规则

遵循 SemVer: 主版本.次版本.修订号

## 发布步骤

1. 在 develop 分支确认所有功能冻结
2. 更新 CHANGELOG.md
3. 提交 PR 合并到 main
4. 打 Tag: `git tag v1.0.0-beta`
5. CI 自动构建并发布:
   - Web 端部署到 Cloudflare Pages
   - 桌面端发布到 GitHub Releases
   - 运行时 CLI 发布到 npm / crates.io

## 回滚策略

- Cloudflare Pages: 在 Dashboard 中选择上一版本回滚
- 桌面端: GitHub Releases 保留旧版本下载链接
