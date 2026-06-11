# 07 - 调试指南

## 调试控制台

在编辑器中按 Ctrl+Shift+D 打开调试控制台。

## 日志过滤

```bash
# 仅显示 VM 模块的 ERROR 级别日志
log filter vm level error

# 开启调试模式
debug on
```

## Wasm 调试

- 浏览器: 使用 Chrome DevTools 的 Wasm 调试支持
- 桌面端: 使用 lldb 或 gdb 附加到扩展进程

## 远程调试桌面端

```bash
# 启动 Tauri 开发模式，启用远程调试
cargo tauri dev -- --remote-debugging-port=9222
```

然后在 Chrome 中访问 `chrome://inspect`。
