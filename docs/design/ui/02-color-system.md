# 02 - 色彩体系

[返回 README](../README.md)

## 1. 概述

Sailfish Studio 的色彩体系围绕单一强调色 (蓝色) 构建，以中性色为主，确保界面安静、专业、不抢内容。状态色仅用于传达功能意义。

## 2. 强调色

### 2.1 主蓝色

用于按钮、链接、选中状态、聚焦环、积木连接点高亮。

| Token | 值 | 用途 |
|-------|----|------|
| `--color-primary` | #2563EB | 主强调色：按钮背景、选中项、链接 |
| `--color-primary-hover` | #1D4ED8 | 悬停状态 |
| `--color-primary-active` | #1E40AF | 按下状态 |
| `--color-primary-light` | #DBEAFE | 浅蓝背景：选中行、信息提示背景 |

### 2.2 深色模式下的蓝色

| Token | 值 | 用途 |
|-------|----|------|
| `--color-primary-dark` | #3B82F6 | 深色模式主强调色 (略亮以保持对比) |
| `--color-primary-dark-hover` | #2563EB | 深色模式悬停 |
| `--color-primary-dark-active` | #1D4ED8 | 深色模式按下 |

## 3. 中性色

### 3.1 浅色模式

| Token | 值 | 用途 |
|-------|----|------|
| `--bg-primary` | #FAFAFA | 主背景 (窗口背景) |
| `--bg-surface` | #FFFFFF | 卡片/面板背景 |
| `--bg-elevated` | #FFFFFF | 弹窗/下拉菜单背景 |
| `--text-primary` | #212121 | 主要文本 |
| `--text-secondary` | #757575 | 辅助文本、图标 |
| `--text-disabled` | #9E9E9E | 禁用文本 |
| `--border-color` | #E0E0E0 | 边框、分隔线 |

### 3.2 深色模式

| Token | 值 | 用途 |
|-------|----|------|
| `--bg-primary-dark` | #121212 | 主背景 |
| `--bg-surface-dark` | #1E1E1E | 卡片/面板背景 |
| `--bg-elevated-dark` | #2A2A2A | 弹窗/下拉菜单背景 |
| `--text-primary-dark` | #E0E0E0 | 主要文本 |
| `--text-secondary-dark` | #9E9E9E | 辅助文本 |
| `--border-color-dark` | #2A2A2A | 边框 |

## 4. 状态色

| Token | 值 | 用途 |
|-------|----|------|
| `--color-error` | #EF4444 | 错误、停止按钮 |
| `--color-success` | #22C55E | 成功、运行状态 |
| `--color-warning` | #F59E0B | 警告、未保存更改 |

## 5. 积木类别色

积木按类别着色，帮助用户快速识别积木类型。这些颜色在浅色和深色模式下保持不变，仅调整明度以保持对比度。

| 类别 | 值 | 说明 |
|------|----|------|
| Motion | #4C97FF | 运动类积木 |
| Looks | #9966FF | 外观类积木 |
| Sound | #CF63CF | 声音类积木 |
| Events | #DE9E2C | 事件类积木 |
| Control | #FFAB19 | 控制类积木 |
| Sensing | #5CB1D6 | 侦测类积木 |
| Operators | #59C059 | 运算类积木 |
| Variables | #FF8C1A | 变量和列表 |
| My Blocks | #FF6680 | 自定义积木 |
