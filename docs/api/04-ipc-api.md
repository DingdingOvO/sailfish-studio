# 04 - IPC 协议完整参考

> 对应设计文档：`docs/design/architecture/02-ipc-protocol.md`

## 协议格式

JSON-RPC 2.0。消息序列化采用 JSON（低数据量）和 MessagePack（高数据量，如绘制指令）混合策略。

```json
{ "jsonrpc": "2.0", "id": "1", "method": "method/name", "params": {} }
```

## 渲染进程 → 扩展进程

| 方法 | 参数 | 返回 | 说明 |
|------|------|------|------|
| project/open | { projectId, path } | { status } | 打开项目 |
| project/close | { projectId } | { status } | 关闭项目 |
| project/compile | { projectId } | { jsCode } | 编译项目 |
| project/run | { projectId } | { status } | 开始执行 |
| project/stop | { projectId } | { status } | 停止执行 |
| block/update | { blockId, field, value } | { status } | 更新积木 |
| variable/set | { varId, value } | { status } | 设置变量 |
| extension/install | { extensionId, source } | { status } | 安装扩展 |

## 扩展进程 → 渲染进程

| 方法 | 参数 | 编码 | 说明 |
|------|------|------|------|
| render/commands | { commands: [] } | MessagePack | 绘制指令列表 |
| project/state | { variables, lists, clones } | JSON | 状态更新 |
| log/entry | { level, source, type, message } | JSON | 日志条目 |
| error/report | { code, message, stack } | JSON | 错误报告 |
| extension/ready | { extensionId } | JSON | 扩展加载完成 |
