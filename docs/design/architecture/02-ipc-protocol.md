# 02 - 进程间通信协议

[返回 README](../README.md)

## 1. 概述

所有进程间通信通过 JSON-RPC 2.0 协议进行。消息序列化采用 JSON (低数据量) 和 MessagePack (高数据量，如绘制指令) 混合策略。

## 2. 传输层

- 渲染进程 ↔ 主进程: `window.__TAURI_INVOKE__` (桌面) 或 `postMessage` (Web)。
- 渲染进程 ↔ 扩展进程: 管道 (桌面 sidecar) 或 `postMessage` (Web Worker)。
- 主进程 ↔ 扩展进程: 管道 (通过主进程中转)。

## 3. 消息格式

```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "project/compile",
  "params": { "projectId": "abc" }
}
```

响应:

```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": { "jsCode": "..." }
}
```

## 4. 核心方法定义

### 4.1 渲染进程 → 扩展进程

- `project/open`: 打开项目，传递 .sf 文件路径。
- `project/compile`: 请求编译项目。
- `project/run`: 开始执行。
- `project/stop`: 停止执行。
- `block/update`: 更新积木参数。
- `variable/set`: 修改变量值。
- `extension/install`: 安装扩展。

### 4.2 扩展进程 → 渲染进程

- `render/commands`: 每帧绘制指令列表 (MessagePack 编码)。
- `project/state`: 变量、列表、克隆体状态更新。
- `log/entry`: 日志条目 (分级)。
- `error/report`: 错误报告。
- `extension/ready`: 扩展加载完成。

## 5. 性能要求

- IPC 延迟 (单条消息): 桌面端 < 1ms，Web 端 < 2ms。
- 绘制指令批量传输: 每帧一次，MessagePack 编码，压缩比 > 1:5。
