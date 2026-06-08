# 05 - 协作 API 参考

> 对应设计文档：`docs/design/collaboration/01-protocol.md`

## WebSocket 连接

```
wss://collab.sailfish.studio/room/{roomId}?token={authToken}
```

## 客户端 → 服务器

| 消息类型 | 说明 |
|----------|------|
| join | 加入房间 |
| leave | 离开房间 |
| operation | 提交操作 (OT)，包含以下子类型：block_insert, block_delete, block_move, block_update_field, variable_set, list_insert, list_delete, sprite_create, sprite_delete, sprite_update_property |
| cursor_move | 光标位置更新 |
| selection_change | 选中内容变化 |
| ping | 心跳 |

## 服务器 → 客户端

| 消息类型 | 说明 |
|----------|------|
| user_joined | 用户加入 |
| user_left | 用户离开 |
| operation_ack | 操作确认 |
| operation_broadcast | 操作广播 |
| cursor_broadcast | 光标位置广播 |
| presence_update | 用户状态更新 |
| conflict | 不可自动解决的冲突通知（如离线编辑后恢复连接） |

## REST API

| 端点 | 方法 | 说明 |
|------|------|------|
| /rooms | POST | 创建房间 |
| /rooms/{id} | GET | 获取房间信息 |
| /rooms/{id}/invite | POST | 生成邀请链接 |
| /rooms/{id}/members | GET | 获取成员列表 |
| /rooms/{id}/members/{uid} | DELETE | 移除成员 |
