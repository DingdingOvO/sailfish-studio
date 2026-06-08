# 08 - 发布流程

## 概述

本文档定义 Sailfish Studio 的版本号规则、发布步骤、回滚策略和发布清单。所有发布必须严格按照本流程执行，确保产品质量和可追溯性。

> 对应设计文档：`docs/design/testing/02-performance-bench.md`
> 对应运维文档：`docs/operations/01-deployment-guide.md`

## 版本号规则

遵循 [SemVer](https://semver.org/) 2.0：

```
主版本.次版本.修订号[-预发布标识]
```

| 版本类型 | 格式 | 示例 | 说明 |
|---------|------|------|------|
| 主版本 | `X.0.0` | `2.0.0` | 不兼容的 API 变更 |
| 次版本 | `1.X.0` | `1.1.0` | 向后兼容的功能新增 |
| 修订号 | `1.0.X` | `1.0.1` | 向后兼容的 Bug 修复 |
| 预发布 | `1.0.0-X.Y` | `1.0.0-beta.1` | 预发布版本 |
| 候选版本 | `1.0.0-rc.X` | `1.0.0-rc.1` | 发布候选 |

## 发布步骤

### 1. 功能冻结

- 在 `develop` 分支确认所有计划功能已完成
- 运行完整测试套件，确保全绿：

```bash
cargo nextest run          # Rust 测试
cargo tarpaulin            # 覆盖率检查 ≥ 95%
pnpm test                  # TypeScript 测试
pnpm e2e                   # E2E 测试
cargo audit                # 安全审计
cargo fuzz run fuzz_target -- -max_total_time=86400  # 模糊测试 24 小时
```

### 2. 创建发布分支

```bash
git checkout develop
git pull origin develop
git checkout -b release/1.0.0
```

### 3. 更新版本号和文档

- 更新所有 `Cargo.toml` 中的版本号
- 更新 `package.json` 中的版本号
- 更新 `CHANGELOG.md`，记录所有变更
- 更新 `docs/requirements/14-changelog.md`

### 4. 测试与验证

- 在所有目标平台上运行 E2E 测试
- 验证性能基准达标（详见 `docs/design/testing/02-performance-bench.md`）
- 检查文档是否与代码一致
- 代码签名验证

### 5. 合并到 main

```bash
# 提交 PR 到 main
gh pr create --base main --head release/1.0.0 --title "Release 1.0.0"

# 合并后打 Tag
git checkout main
git pull origin main
git tag -a v1.0.0 -m "Release 1.0.0"
git push origin v1.0.0
```

### 6. CI 自动构建与发布

打 Tag 后，GitHub Actions 自动执行：

| 产物 | 部署目标 | 说明 |
|------|---------|------|
| Web 端 | Cloudflare Pages | `editor.sailfish.studio` |
| 桌面端 .dmg | GitHub Releases | macOS (Apple Silicon + Intel) |
| 桌面端 .msi | GitHub Releases | Windows |
| 桌面端 .AppImage | GitHub Releases | Linux |
| CLI 运行时 | crates.io + npm | `sf-runtime` |
| 文档站 | Cloudflare Pages | `docs.sailfish.studio` |

### 7. 构建来源证明

- GitHub Actions 生成 SLSA Level 2+ 构建来源证明
- 用户可验证发布包是否由官方 CI 构建
- 构建配置和依赖锁定文件在 Git 中可审计

## 回滚策略

### Web 端（Cloudflare Pages）

1. 登录 Cloudflare Dashboard
2. 选择 Pages 项目
3. 在 "Deployments" 中选择上一版本
4. 点击 "Rollback to this deployment"
5. 回滚在数秒内生效

### 桌面端（GitHub Releases）

1. GitHub Releases 保留所有历史版本的下载链接
2. 如果新版本有严重 Bug，在 Release Notes 中添加警告
3. Tauri 自动更新会检查 `update.json`，修改该文件指向稳定版本
4. 用户可手动下载旧版本安装

### CLI 运行时

1. `crates.io` 和 `npm` 不支持删除已发布版本
2. 如果有严重 Bug，发布修补版本（修订号 +1）
3. 使用 `cargo yank` 阻止新安装（已安装的不受影响）

## 发布清单

发布前逐项确认：

- [ ] 所有 CI 检查全绿
- [ ] 测试覆盖率 ≥ 95%
- [ ] 模糊测试 24 小时无崩溃
- [ ] 性能基准全部达标
- [ ] CHANGELOG.md 已更新
- [ ] 版本号已更新（Cargo.toml + package.json）
- [ ] 代码签名完成（macOS + Windows + Linux）
- [ ] 构建来源证明已生成
- [ ] 文档与代码一致
- [ ] 至少 2 人在目标平台手动验证
