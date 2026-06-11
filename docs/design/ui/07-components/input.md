# 输入框组件规范

[上一节：按钮](./button.md) | [下一节：弹窗](./dialog.md)

## 1. 变体

### 1.1 单行输入框

- 高度: 32px
- 内边距: `--space-2` (8px) 水平
- 圆角: 6px
- 边框: 1px `--border-color`
- 背景: `--bg-surface`
- 文字: `--text-primary`, 字号 `--text-base`

### 1.2 多行文本框

- 最小高度: 80px
- 内边距: `--space-3` (12px)
- 支持手动调整大小 (右下角手柄)。

### 1.3 带标签的输入框

- 标签位于输入框上方，间距 `--space-1` (4px)。
- 标签字号 `--text-sm`, 颜色 `--text-secondary`。

## 2. 状态

- **默认**: 边框 `--border-color`。
- **悬停**: 边框变为 `--color-primary-light`。
- **聚焦**: 边框变为 `--color-primary`, 外发光 0 0 0 2px `--color-primary-light`。
- **错误**: 边框变为 `--color-error`, 输入框下方显示错误提示文本。
- **禁用**: 背景变为 `--bg-primary` (浅灰), 文字 `--text-disabled`, 不可交互。

## 3. 校验规则

- 输入验证在失焦时触发，不在每次按键时触发 (避免打断输入)。
- 类型化变量输入框: 根据变量类型实时过滤非法字符 (如 number 类型输入框不接受字母)。
