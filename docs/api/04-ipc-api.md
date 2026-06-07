# 04 - IPC 协议完整参考

## 协议格式

JSON-RPC 2.0:

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

## 扩展进程 → 渲染进程

| 方法 | 参数 | 说明 |
|------|------|------|
| render/commands | { commands: [] } | 绘制指令列表 |
| project/state | { variables, lists, clones } | 状态更新 |
| log/entry | { level, source, type, message } | 日志条目 |
| error/report | { code, message, stack } | 错误报告 |
