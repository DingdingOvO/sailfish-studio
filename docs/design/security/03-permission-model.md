# 03 Permission Model

03 - 权限模型

1. 概述

扩展和插件必须声明所需权限，用户安装时确认。权限不可在运行时动态请求。

2. 权限声明 (extension.json / plugin.json)

```json
{
  "permissions": [
    "network:api.example.com",
    "storage",
    "audio"
  ]
}
```

3. 权限列表

· network / network:<domain>: 网络请求，可限制域名
· storage: 本地存储读写
· filesystem: 文件系统访问 (仅桌面端，限制在项目目录和用户数据目录)
· audio: 音频播放和控制
· video: 视频播放和控制
· dom: DOM 访问 (仅插件，限制在指定 UI 区域)
· vm: sf-vm 内部状态访问 (需额外审核，仅限官方认证扩展)

4. 权限确认 UI

· 安装扩展/插件时弹出权限确认对话框，列出所有权限及说明
· 用户必须逐项勾选同意，不能批量同意
· 权限不可在运行时动态请求
