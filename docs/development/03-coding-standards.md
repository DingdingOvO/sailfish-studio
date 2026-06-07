# 03 - 编码规范

## Rust 规范

- 格式化: `cargo fmt --check`
- Lint: `cargo clippy -- -D warnings`
- 禁止 `unsafe` (除非审查批准并附注释)
- 禁止 `unwrap()` / `expect()` (使用 `?` 或显式错误处理)
- 禁止魔法数字 (必须定义为具名常量)
- 所有公共 API 必须有文档注释 `///`
- 测试覆盖率 ≥ 95%

## TypeScript 规范

- 格式化 & Lint: Biome
- 禁止 `any` 类型 (除非审查批准并附注释)
- 禁止魔法数字
- 所有公共函数必须有 JSDoc
- 测试覆盖率 ≥ 95%

## 命名规范

- Rust: snake_case 函数/变量, CamelCase 类型, SCREAMING_SNAKE_CASE 常量
- TypeScript: camelCase 函数/变量, PascalCase 类/组件, SCREAMING_SNAKE_CASE 常量
