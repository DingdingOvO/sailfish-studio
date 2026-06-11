# 03 - 字体与排版

[返回 README](../README.md)

## 1. 字体族

### 1.1 系统 UI 字体

针对不同操作系统优化，确保原生渲染效果和最佳可读性。

| 平台 | 字体栈 | 说明 |
|------|--------|------|
| macOS / iOS | 'SF Pro Text', -apple-system, BlinkMacSystemFont, sans-serif | Apple 系统原生 UI 字体，经过光学尺寸优化 |
| Windows | 'Segoe UI', system-ui, sans-serif | Windows 现代 UI 字体，对低分辨率屏幕有 hinting 优化 |
| Linux | 'Inter', system-ui, sans-serif | 跨平台开源字体，字形清晰 |
| Android | 'Roboto', system-ui, sans-serif | Android 原生 UI 字体 |

### 1.2 等宽字体

用于代码编辑器、.sfl 文本编辑、调试控制台、变量监视器中的数值显示。

| 平台 | 字体栈 | 说明 |
|------|--------|------|
| 通用 | 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Cascadia Code', monospace | JetBrains Mono 具有优秀的连字支持和可区分的字符设计 (如 0 和 O、1 和 l) |

## 2. 字号体系 (Type Scale)

使用 Major Third (1.25) 比例，以 13px 为基准。

| Token | 字号 | 行高 | 用途 |
|-------|------|------|------|
| `--text-xs` | 11px | 16px | 最小文本：状态栏、键盘快捷键提示、标签 |
| `--text-sm` | 13px | 20px | 辅助文本：描述、占位符、面板标题 |
| `--text-base` | 16px | 24px | 正文：菜单项、列表项、设置表单文本 |
| `--text-md` | 20px | 28px | 中等标题：卡片标题、弹窗标题 |
| `--text-lg` | 25px | 32px | 大标题：面板标题、页面标题 |
| `--text-xl` | 31px | 38px | 特大标题：欢迎页、教程页 |
| `--text-code` | 13px | 20px | 代码/等宽文本 |

## 3. 字重

| Token | 值 | 用途 |
|-------|----|------|
| `--weight-regular` | 400 | 正文文本 |
| `--weight-medium` | 500 | 标签、按钮文本 |
| `--weight-semibold` | 600 | 副标题、面板标题 |
| `--weight-bold` | 700 | 页面标题、强调内容 |

## 4. 排版规则

- **行长限制**: 正文文本行长不超过 75 字符 (约 600px)，以保持阅读舒适度。标题不受此限制。
- **段落间距**: 段落间间距为 1em，标题与正文间间距为 0.5em。
- **数字与等宽**: 变量值、监视器数值、代码片段统一使用等宽字体，数字右对齐以确保数位对齐。
- **文本截断**: 长文本在不支持换行的区域 (如积木标签) 使用省略号截断，并提供 tooltip 显示完整文本。截断阈值由组件宽度和平均字符宽度计算，不在 CSS 中用魔法数字。
