# 按钮组件规范

[返回 README](../../README.md) | [上一节：动画规范](../06-motion.md) | [下一节：输入框](./input.md)

## 1. 变体

### 1.1 主要按钮 (Primary)

用于主要操作 (如"保存"、"运行")。

- 背景: `--color-primary`
- 文字: 白色
- 圆角: 6px
- 内边距: `--space-2` (8px) 水平, `--space-1` (4px) 垂直
- 最小宽度: 64px
- 高度: 32px
- 状态: 悬停 (`--color-primary-hover`)、按下 (`--color-primary-active`)、禁用 (灰度，30% 不透明度)

### 1.2 次要按钮 (Secondary)

用于辅助操作 (如"取消"、"重置")。

- 背景: 透明
- 边框: 1px `--border-color`
- 文字: `--text-primary`
- 其他尺寸同主要按钮。

### 1.3 危险按钮 (Danger)

用于不可逆操作 (如"删除")。

- 背景: `--color-error`
- 文字: 白色

### 1.4 图标按钮 (Icon-only)

用于工具栏。

- 尺寸: 32px × 32px
- 背景: 透明
- 悬停: `--bg-surface` 加 8% 不透明叠加
- 图标: 20px, `--text-secondary`
- 必须有 Tooltip 描述功能。

## 2. 交互规范

- 按钮必须支持 Enter/Space 键盘激活。
- 聚焦时显示 2px 蓝色轮廓 (`--color-primary`)。
