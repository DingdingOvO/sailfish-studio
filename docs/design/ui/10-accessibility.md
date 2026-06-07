# 无障碍设计规范

[返回 README](../README.md)

## 1. 键盘导航

- 所有交互元素 (按钮、菜单项、输入框、树节点) 可通过 Tab/Shift+Tab 聚焦。
- 聚焦环: 2px 蓝色 (`--color-primary`) 外框，与元素边缘间距 2px。
- Tab 顺序: 工具栏 → 工具箱 → 积木画布 → 舞台 → 右侧面板 → 状态栏。
- 积木画布内: ↑↓←→ 在积木连接点间移动，Enter 选中/连接。

## 2. 屏幕阅读器

- 所有组件提供 ARIA 标签 (`aria-label`, `role`)。
- 舞台角色: `role="img"`, `aria-label="角色名称"`。
- 积木: `role="button"`, `aria-label="积木类型: 移动10步"`。

## 3. 颜色对比度

- 所有文本与背景对比度 ≥ 4.5:1 (WCAG AA)。
- 大文本 (≥18px 或 14px 粗体) 对比度 ≥ 3:1。

## 4. 减少动画

- 用户系统设置 `prefers-reduced-motion: reduce` 时，所有动画持续时间降为 0ms，仅保留必要的状态过渡 (如焦点指示器)。
- 动态模糊拖尾在 `prefers-reduced-motion` 时关闭。
