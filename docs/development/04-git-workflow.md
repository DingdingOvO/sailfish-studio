# 04 - Git 工作流与分支策略

## 分支策略

- `main`: 稳定发布分支
- `develop`: 开发分支
- `feature/<name>`: 功能分支 (从 develop 创建)
- `fix/<name>`: 修复分支

## Commit Message 规范

遵循 Conventional Commits:

```
<type>(<scope>): <description>
```

类型: feat, fix, docs, style, refactor, test, chore

## PR 流程

1. 创建 feature 分支
2. 开发并自测
3. 提交 PR 到 develop
4. CI 自动检查 (格式、lint、测试、覆盖率)
5. 至少一人 Code Review 并批准
6. 合并到 develop
