# 11 - 非功能性需求

[上一章](./10-debug.md) | [下一章：路线图](./12-roadmap.md)

## 11.1 性能
- 1000 积木编译 < `COMPILE_TIME_TARGET_MS`
- 重负载帧率 ≥ `FPS_MIN_ACCEPTABLE` fps，波动 ≤ `FPS_FLUCTUATION_MAX` fps
- 编辑拖拽波动 ≤ 3 fps
- IPC 延迟：桌面 < 1ms，浏览器 < 2ms
- 空项目 Wasm 内存 < `WASM_HEAP_BUDGET_DEFAULT_MB / 4`
- 大型项目 Wasm 内存 < `WASM_HEAP_BUDGET_DEFAULT_MB`

## 11.2 安全
扩展审核、沙箱隔离、进程隔离、CSP、代码签名。

## 11.3 兼容性
Chrome 90+、Firefox 90+、Safari 15+。Windows 10+、macOS 11+、Linux。Android 10+。

## 11.4 可维护性
测试覆盖率 ≥ 95%。所有模块文档化。禁止魔法数字。

## 11.5 国际化
50 种语言。AI 翻译。日志始终英文。
