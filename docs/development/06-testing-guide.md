# 06 - 测试指南

## 测试类型

| 类型 | 工具 | 命令 |
|------|------|------|
| Rust 单元测试 | cargo-nextest | `cargo nextest run` |
| Rust 覆盖率 | cargo-tarpaulin | `cargo tarpaulin` |
| TS 单元测试 | Vitest | `pnpm test` |
| E2E 测试 | Playwright | `pnpm e2e` |
| 模糊测试 | cargo-fuzz | `cargo fuzz run fuzz_target` |

## 编写测试

- 单元测试放在 `tests/` 目录
- 测试数据放在 `test_data/` 目录
- 测试函数命名: `test_<模块>_<场景>_<预期结果>`
