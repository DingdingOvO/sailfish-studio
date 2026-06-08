# 07 - 调试指南

## 概述

Sailfish Studio 的多进程架构决定了调试方式因进程和平台而异。本文档覆盖编辑器调试控制台、WASM 调试、Rust 子进程调试、以及远程调试等场景。

> 对应需求文档：`docs/requirements/10-debug.md`
> 对应设计文档：`docs/design/architecture/01-process-model.md`

## 调试控制台

在编辑器中按 `Ctrl+Shift+D`（macOS: `Cmd+Shift+D`）打开调试控制台。调试控制台是开发者最常用的调试工具，集成了日志查看、状态检查和性能分析功能。

### 功能

| 功能 | 说明 |
|------|------|
| 日志过滤 | 按级别（TRACE/DEBUG/INFO/WARN/ERROR/FATAL）、模块名、关键词过滤 |
| 日志搜索 | 全文搜索日志内容，支持正则表达式 |
| 日志导出 | 导出为 JSON 或纯文本文件，方便附加到 Bug 报告 |
| 性能快照 | 捕获当前帧的渲染指令、WASM 内存使用、克隆体数量 |
| 调用栈查看 | 查看当前执行中的积木调用栈 |
| 进程状态 | 显示所有扩展进程的运行状态、内存占用、CPU 使用率 |

### 日志命令

调试控制台支持内置命令：

```
log filter <module> level <level>   # 过滤日志
debug on                            # 开启调试模式（日志级别降至 DEBUG）
debug off                           # 关闭调试模式
perf snapshot                       # 捕获性能快照
state dump                          # 导出当前项目状态
process list                        # 列出所有扩展进程
```

### 日志级别

正常模式下仅记录 WARN 及以上级别，调试模式记录 DEBUG 及以上。详见 `docs/requirements/00-constants.md` 中的日志常量定义。

| 级别 | 值 | 正常模式 | 调试模式 |
|------|---|---------|---------|
| TRACE | 0 | 不记录 | 不记录 |
| DEBUG | 1 | 不记录 | 记录 |
| INFO | 2 | 不记录 | 记录 |
| WARN | 3 | 记录 | 记录 |
| ERROR | 4 | 记录 | 记录 |
| FATAL | 5 | 记录 | 记录 |

## WebAssembly 调试

### 浏览器端

Chrome DevTools 提供了 WASM 调试支持：

1. 打开 Chrome DevTools → Sources 面板
2. 在左侧文件树中找到 WASM 模块（通常以 `sf_vm_bg.wasm` 命名）
3. 设置断点、单步执行、查看变量

> **注意**：WASM 调试需要编译 Debug 版本（不含 `wasm-opt` 优化），否则断点可能无法命中。

```bash
# 构建 Debug 版本用于调试
wasm-pack build sf-vm --target web --dev
```

### 常见 WASM 调试技巧

- **内存泄漏**：使用 Chrome Memory 面板，对比堆快照（Heap Snapshot），检查 `sf_vm_create` 后是否有对应 `sf_vm_destroy`
- **性能瓶颈**：使用 Chrome Performance 面板录制，查看 WASM 函数耗时
- **Panic 追踪**：确保 `console_error_panic_hook` 已启用，panic 时会在控制台输出完整的 Rust 调用栈

## 桌面端调试

### Rust 子进程调试

扩展进程在桌面端作为独立子进程运行，可以使用系统调试器附加：

```bash
# 使用 lldb 附加到扩展进程（macOS/Linux）
lldb -p <extension_process_pid>

# 使用 gdb 附加（Linux）
gdb -p <extension_process_pid>

# Windows：使用 Visual Studio 附加到进程
```

### 远程调试 Tauri WebView

```bash
# 启动 Tauri 开发模式，启用远程调试端口
cargo tauri dev -- --remote-debugging-port=9222
```

然后在 Chrome 中访问 `chrome://inspect`，点击 "Configure..." 添加 `localhost:9222`，即可看到 Tauri WebView 页面并进行调试。

### Tauri 日志

```bash
# 启动时设置 RUST_LOG 环境变量控制日志级别
RUST_LOG=debug cargo tauri dev

# 仅查看特定模块的日志
RUST_LOG=sf_vm=debug cargo tauri dev
```

## 性能调试

### 帧率问题

1. 打开调试控制台，查看性能仪表盘
2. 检查 WASM 内存使用是否接近 `WASM_HEAP_BUDGET_DEFAULT_MB` (256MB)
3. 检查克隆体数量是否超过 `CLONE_DEFAULT_MAX` (300)
4. 使用 `perf snapshot` 捕获性能快照
5. 检查 IPC 消息量是否异常（大量绘制指令可能表示渲染问题）

### 内存问题

1. 使用 Chrome Memory 面板（Web 端）或 Instruments/VTune（桌面端）
2. 检查是否有未释放的扩展进程
3. 检查日志缓冲区是否超过 `LOG_BUFFER_MEMORY_MAX_MB` (16MB)
4. 检查纹理缓存是否超过 `TEXTURE_CACHE_MAX_COUNT` (256)

### IPC 延迟问题

1. 检查 IPC 消息大小是否合理
2. 确认 MessagePack 编码已启用（绘制指令等大数据量消息）
3. 桌面端检查 sidecar 进程是否正常运行
4. Web 端检查 Web Worker 是否被浏览器限制

## 多进程调试策略

由于 Sailfish Studio 采用三进程架构，调试时需要注意：

| 进程 | 调试方式 | 注意事项 |
|------|---------|---------|
| 主进程 | RUST_LOG + lldb/gdb | 桌面端 Tauri Rust 后端 |
| 渲染进程 | Chrome DevTools | 前端 UI 代码 |
| 扩展进程 | WASM 调试 + 日志 | 每个项目独立进程 |

### 进程崩溃恢复

- 渲染进程每 5 秒向扩展进程发送心跳
- 若 15 秒无响应，判定为崩溃
- 崩溃时扩展进程的日志缓冲区（环形缓冲）被写入崩溃报告
- 用户可选择恢复：重新创建扩展进程并加载最近的自动保存快照
