# 03 - 编码规范

## 概述

本文档定义 Sailfish Studio 项目的编码规范，所有贡献者必须严格遵守。规范覆盖 Rust、TypeScript 两大核心语言，以及命名、文件组织等通用约定。CI 会在每次提交时自动检查格式和 lint，不合规的代码将被拒绝合并。

> 对应需求文档：`docs/requirements/03-architecture.md` Section 3.3

## Rust 规范

### 格式化与 Lint

- **格式化**：使用 `cargo fmt --check` 检查，所有代码必须通过 `rustfmt` 格式化。禁止使用 `#[rustfmt::skip]` 除非有充分理由并在 PR 中说明。
- **Lint**：使用 `cargo clippy -- -D warnings`，所有 clippy 警告视为错误。禁止使用 `#[allow(clippy::...)]` 除非有充分理由并在 PR 中说明。
- **代码审查**：所有 PR 必须至少一人 Code Review 并批准，CI 必须全绿。

### 安全性约束

- **禁止 `unsafe`**：生产代码中不得使用 `unsafe` 块，除非通过安全审查并在注释中说明为何必要、如何保证安全。`unsafe` 代码必须附带 `// SAFETY:` 注释。
- **禁止 `unwrap()` / `expect()`**：生产代码中不得使用 `unwrap()` 或 `expect()`，必须使用 `?` 运算符或显式错误处理。测试代码中可以使用 `unwrap()`。
- **禁止魔法数字**：代码中不得出现裸数字，所有数值必须定义为具名常量。参考 `docs/requirements/00-constants.md` 中的常量索引。

### 文档与测试

- **公共 API 文档**：所有 `pub` 函数、结构体、枚举、trait 必须有 `///` 文档注释，说明参数、返回值、可能抛出的错误。
- **测试覆盖率 ≥ 95%**：使用 `cargo tarpaulin` 生成覆盖率报告。所有公共 API 必须有单元测试，所有 IPC 方法必须有集成测试。

### 错误处理

- 使用 `thiserror` 定义自定义错误类型，不使用 `Box<dyn Error>`。
- 错误类型必须实现 `Display` 和 `Error` trait。
- WASM 导出函数中的错误必须转换为 `JsValue` 返回给 JavaScript。

```rust
// 正确：使用 thiserror 定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("项目加载失败: {0}")]
    LoadFailed(String),
    #[error("编译错误: {0}")]
    CompileError(String),
    #[error("运行时错误: {0}")]
    RuntimeError(String),
}

// 正确：WASM 导出中的错误转换
#[wasm_bindgen]
pub fn sf_vm_compile(state_ptr: *mut RuntimeState, project_json: &str) -> Result<String, JsValue> {
    let project: Project = serde_json::from_str(project_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    compile(&project).map_err(|e| JsValue::from_str(&e.to_string()))
}
```

## TypeScript 规范

### 格式化与 Lint

- **格式化 & Lint**：使用 Biome 统一处理。配置文件为 `biome.json`。禁止使用 ESLint 或 Prettier。
- **禁止 `any` 类型**：生产代码中不得使用 `any` 类型，除非通过审查并在注释中说明。使用 `unknown` 替代，并进行类型窄化。
- **禁止魔法数字**：与 Rust 相同，所有数值必须定义为具名常量。
- **严格模式**：`tsconfig.json` 中启用 `strict: true`。

### 文档与测试

- **公共函数文档**：所有导出函数必须有 JSDoc 注释，说明参数、返回值、可能抛出的错误。
- **测试覆盖率 ≥ 95%**：使用 Vitest 生成覆盖率报告。所有公共函数必须有单元测试。

### 状态管理

- 使用 Zustand 管理全局状态，每个 Store 职责单一。
- Store 命名：`use[Name]Store`（如 `useProjectStore`、`useEditorStore`）。
- 禁止在组件中直接修改 Store 外部的全局状态。

## 命名规范

### Rust 命名

| 类型 | 风格 | 示例 |
|------|------|------|
| 函数 / 变量 | snake_case | `move_steps`, `runtime_state` |
| 类型 / 结构体 / 枚举 | CamelCase | `RuntimeState`, `BlockLayout` |
| 常量 | SCREAMING_SNAKE_CASE | `STAGE_MAX_WIDTH`, `TARGET_FPS` |
| Crate | kebab-case | `sf-vm`, `sf-blocks` |
| Trait | CamelCase | `SfExtension`, `BlockRenderer` |
| Module | snake_case | `compiler`, `runtime` |

### TypeScript 命名

| 类型 | 风格 | 示例 |
|------|------|------|
| 函数 / 变量 | camelCase | `loadWasm`, `projectState` |
| 类 / 组件 | PascalCase | `BlockCanvas`, `StageView` |
| 常量 | SCREAMING_SNAKE_CASE | `MAGNETIC_SNAP_DISTANCE_PX` |
| 类型 / 接口 | PascalCase | `Project`, `BlockConfig` |
| 文件名 | kebab-case | `block-canvas.tsx`, `use-project-store.ts` |
| CSS 变量 | kebab-case | `--color-primary`, `--space-4` |

## 通用约定

### 文件组织

- 每个 Rust module 放在独立文件或目录中（`mod.rs` 或同名文件）。
- TypeScript 组件使用 `index.tsx` 导出，样式使用 CSS Modules 或 Tailwind 类名。
- 测试文件放在 `tests/` 目录，与源代码分离。
- 测试数据放在 `test_data/` 或 `__fixtures__/` 目录。

### Git 约定

- Commit Message 遵循 Conventional Commits（详见 `docs/development/04-git-workflow.md`）。
- 每个 PR 只做一件事，避免混合多种变更。
- PR 描述必须包含变更原因、变更内容、测试方法。

### 性能意识

- 避免在热路径（渲染循环、事件处理）中分配内存。使用对象池、预分配缓冲区。
- 所有跨进程通信（IPC）消息尽量精简，避免传输冗余数据。
- WebGL 绘制调用按纹理排序批量提交，减少状态切换。
