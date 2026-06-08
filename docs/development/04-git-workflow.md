# 04 - Git 工作流与分支策略

## 概述

本文档定义 Sailfish Studio 项目的 Git 工作流、分支策略和 Commit 规范。所有贡献者必须遵守这些规则，确保代码仓库的整洁和可追溯性。

> 对应需求文档：`docs/requirements/03-architecture.md` Section 3.3

## 分支策略

Sailfish Studio 采用简化的 Git Flow 模型：

| 分支 | 用途 | 保护规则 |
|------|------|---------|
| `main` | 稳定发布分支，只接受来自 develop 的合并 | 必须 PR + 1 人审查 + CI 全绿 |
| `develop` | 开发集成分支，日常开发的基准 | 必须 PR + CI 全绿 |
| `feature/<name>` | 功能分支，从 develop 创建 | 无 |
| `fix/<name>` | 修复分支，从 develop 或 main 创建 | 无 |
| `release/<version>` | 发布准备分支，从 develop 创建 | 无 |

### 分支生命周期

1. 开发者从 `develop` 创建 `feature/xxx` 分支
2. 在功能分支上开发并自测
3. 提交 PR 到 `develop`，CI 自动检查
4. 至少一人 Code Review 并批准
5. 合并到 `develop`，删除功能分支
6. 达到发布条件时，从 `develop` 创建 `release/x.x.x`
7. 测试通过后合并到 `main`，打 Tag

## Commit Message 规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) v1.0：

```
<type>(<scope>): <description>

[可选的正文]

[可选的脚注]
```

### 类型（Type）

| 类型 | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat(vm): add clone support` |
| `fix` | 修复 Bug | `fix(renderer): fix WebGL context loss` |
| `docs` | 文档变更 | `docs(api): update extension API reference` |
| `style` | 代码格式（不影响逻辑） | `style(editor): fix indentation` |
| `refactor` | 重构（不增加功能、不修复 Bug） | `refactor(parser): extract sb3 parsing` |
| `test` | 测试相关 | `test(vm): add compiler unit tests` |
| `chore` | 构建/工具变更 | `chore(ci): update GitHub Actions` |
| `perf` | 性能优化 | `perf(renderer): batch draw calls` |

### Scope（范围）

Scope 对应仓库中的模块：

- **sf-core crates**：`vm`, `blocks`, `renderer`, `parser`, `storage`, `audio`
- **sf-editor**：`ui`, `editor`, `plugins`, `i18n`
- **sf-runtime**：`cli`, `runtime`
- **全局**：`workspace`, `deps`, `ci`

### 规则

- 描述使用英文，首字母小写，不加句号
- Breaking Change 在脚注中标记：`BREAKING CHANGE: xxx`
- 关联 Issue：在脚注中写 `Closes #123` 或 `Refs #456`

### 示例

```
feat(vm): implement extension settings engine

Add SettingsEngine with multi-layer priority resolution
(user > project > session > defaults) and subscriber notification.

Refs #42
```

```
fix(blocks): correct snap distance calculation

The snap detection was using Manhattan distance instead of
Euclidean distance, causing inconsistent snap behavior.

Closes #89
```

## PR 流程

### 创建 PR

1. 确保功能分支已与 `develop` 同步（`git rebase develop`）
2. 确保本地所有测试通过（`cargo nextest run` + `pnpm test`）
3. 在 GitHub 上创建 Pull Request，目标分支为 `develop`
4. 填写 PR 模板：变更原因、变更内容、测试方法、截图（如适用）

### CI 检查

每次提交 PR 时，CI 自动执行以下检查：

| 检查项 | Rust | TypeScript |
|--------|------|-----------|
| 格式化 | `cargo fmt --check` | `pnpm biome check` |
| Lint | `cargo clippy -- -D warnings` | `pnpm biome lint` |
| 单元测试 | `cargo nextest run` | `pnpm vitest run` |
| 覆盖率 | `cargo tarpaulin` (≥ 95%) | `pnpm vitest --coverage` (≥ 95%) |
| 安全审计 | `cargo audit` | `pnpm audit` |

### Code Review

- 所有 PR 必须至少一人审查并批准
- 审查者应关注：代码正确性、规范性、测试覆盖、性能影响
- 审查意见必须被回复或处理后才能合并
- 作者不得自行批准自己的 PR

### 合并规则

- 使用 Squash Merge 合并到 `develop`，保持提交历史清晰
- 合并到 `main` 时使用 Merge Commit，保留发布历史
- 合并后自动删除源分支

## 版本号规则

遵循 [SemVer](https://semver.org/) 2.0：

- `主版本.次版本.修订号`（如 `1.2.3`）
- 主版本：不兼容的 API 变更
- 次版本：向后兼容的功能新增
- 修订号：向后兼容的 Bug 修复

预发布版本使用后缀：`1.0.0-beta.1`、`1.0.0-rc.1`
