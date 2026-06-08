# 06 - 测试指南

## 概述

Sailfish Studio 的测试体系遵循测试金字塔原则：70% 单元测试、20% 集成测试、10% 端到端测试。总覆盖率要求 ≥ 95%。本文档描述各类测试的编写规范、工具选择和执行方式。

> 对应设计文档：`docs/design/testing/01-test-pyramid.md`
> 对应需求文档：`docs/requirements/11-nfr.md`

## 测试类型

| 类型 | 占比 | 工具 | 命令 | 说明 |
|------|------|------|------|------|
| Rust 单元测试 | 35% | cargo-nextest | `cargo nextest run` | 每个函数独立测试 |
| Rust 集成测试 | 15% | cargo-nextest | `cargo nextest run` | 模块间交互、IPC 协议 |
| TS 单元测试 | 35% | Vitest | `pnpm test` | React 组件、Store、工具函数 |
| TS 集成测试 | 5% | Vitest + MSW | `pnpm test:integration` | API 模拟、WASM 交互 |
| E2E 测试 | 10% | Playwright | `pnpm e2e` | 完整用户流程 |
| 模糊测试 | — | cargo-fuzz | `cargo fuzz run fuzz_target` | 边界情况与安全漏洞 |

## Rust 测试

### 单元测试

- 测试文件放在 `tests/` 目录（集成测试）或 `src/` 目录内的 `#[cfg(test)] mod tests` 块中
- 测试数据放在 `test_data/` 目录
- 测试函数命名：`test_<模块>_<场景>_<预期结果>`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_steps_positive_direction() {
        let mut state = RuntimeState::new();
        state.targets[0].direction = 90.0;
        move_steps(&mut state, 0, 10.0);
        assert_eq!(state.targets[0].x, 10.0);
    }

    #[test]
    fn test_move_steps_zero_steps_no_movement() {
        let mut state = RuntimeState::new();
        let original_x = state.targets[0].x;
        move_steps(&mut state, 0, 0.0);
        assert_eq!(state.targets[0].x, original_x);
    }
}
```

### 集成测试

- 测试 IPC 协议消息的序列化/反序列化
- 测试跨模块交互（如 compiler + runtime）
- 测试 .sf / .sb3 文件的完整加载流程

### 覆盖率

```bash
# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage/

# 检查覆盖率阈值
cargo tarpaulin --out Stdout | grep "Coverage"
```

### 模糊测试

模糊测试针对核心模块发现边界情况和安全漏洞：

```bash
# 运行模糊测试（持续 1 小时）
cargo fuzz run fuzz_parse_sb3 -- -max_total_time=3600

# 运行模糊测试（发布前，持续 24 小时）
cargo fuzz run fuzz_parse_sb3 -- -max_total_time=86400
```

模糊测试目标：
- **sf-parser**：随机字节输入，验证不会 panic
- **sf-vm compiler**：随机 AST 输入，验证生成的代码合法
- **协作协议**：随机操作序列，验证 OT 变换结果一致

详见 `docs/design/testing/04-fuzzing-strategy.md`。

## TypeScript 测试

### 单元测试

使用 Vitest 编写，放在 `tests/unit/` 目录：

```typescript
import { describe, it, expect } from 'vitest';
import { useProjectStore } from '@/lib/stores/use-project-store';

describe('useProjectStore', () => {
  it('should set project name', () => {
    const store = useProjectStore.getState();
    store.setProjectName('Test Project');
    expect(store.projectName).toBe('Test Project');
  });
});
```

### 组件测试

使用 @testing-library/react 测试 React 组件：

```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { BlockCanvas } from '@/components/block-canvas';

describe('BlockCanvas', () => {
  it('should render blocks', () => {
    render(<BlockCanvas blocks={mockBlocks} />);
    expect(screen.getByText('移动 10 步')).toBeInTheDocument();
  });
});
```

### E2E 测试

使用 Playwright 编写端到端测试，放在 `tests/e2e/` 目录：

```typescript
import { test, expect } from '@playwright/test';

test('user can create and run a project', async ({ page }) => {
  await page.goto('http://localhost:3000');
  await page.click('text=新建');
  await page.dragAndDrop('[data-testid="motion-move-steps"]', '[data-testid="block-canvas"]');
  await page.click('[data-testid="green-flag"]');
  await expect(page.locator('[data-testid="stage"]')).toBeVisible();
});
```

### 无障碍测试

使用 @axe-core/playwright 进行无障碍测试：

```typescript
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('accessibility check', async ({ page }) => {
  await page.goto('http://localhost:3000');
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
```

## CI 集成

### GitHub Actions 测试矩阵

| OS | Rust | Node | Browser |
|----|------|------|---------|
| ubuntu-latest | stable, beta | 24 LTS | Chromium, Firefox, WebKit |
| macos-latest | stable | 24 LTS | Chromium, WebKit |
| windows-latest | stable | 24 LTS | Chromium |

### 测试执行

```bash
# 完整测试套件
cargo nextest run           # Rust 单元 + 集成测试
cargo tarpaulin             # Rust 覆盖率
pnpm test                   # TS 单元测试
pnpm test:integration       # TS 集成测试
pnpm e2e                    # Playwright E2E
cargo audit                 # 安全审计
```

## 性能基准

详见 `docs/design/testing/02-performance-bench.md`。

| 指标 | 目标 | 测试工具 |
|------|------|---------|
| 1000 积木编译 | < 200ms | criterion |
| 1000 角色渲染 | ≥ 60fps | Playwright 性能录制 |
| IPC 单条延迟（桌面） | < 1ms | 微基准测试 |
| IPC 单条延迟（Web） | < 2ms | 微基准测试 |
