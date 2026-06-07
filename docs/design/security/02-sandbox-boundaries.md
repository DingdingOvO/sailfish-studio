# 02 Sandbox Boundaries

02 - 沙箱边界

1. 概述

Sailfish Studio 的沙箱边界定义扩展/插件与系统资源的隔离方式。

2. 边界定义

2.1 DOM 隔离

· 扩展无权访问 DOM，所有 UI 操作通过 API 代理。尝试直接访问 document 或 window 时返回 undefined。
· 插件可访问编辑器 UI 的指定区域 (sf-editor 的 <div id="plugin-panel">)，但不能访问其他区域 (菜单、工具栏、舞台)。

2.2 存储隔离

· 每个扩展拥有独立的 IndexedDB/LocalStorage 命名空间，键名自动添加扩展 ID 前缀
· 跨扩展数据共享必须通过 sailfish.storage.share(key, value, targetExtensionId) API

2.3 网络隔离

· 扩展的网络请求通过编辑器主线程的代理执行，不在扩展进程中直接发起
· 代理检查请求 URL 是否在扩展声明的域名白名单中，拒绝非白名单请求
· 请求速率限制: 60 次/分钟，超出后返回 429

2.4 进程隔离 (桌面端)

· 每个扩展在独立子进程中运行，通过 IPC 通信。扩展进程崩溃不影响编辑器和其他扩展。
· 子进程使用操作系统级沙箱 (macOS App Sandbox、Windows Job Objects)，限制文件系统和网络访问。
· 扩展进程内存限制 128MB，超出时进程被终止并提示用户。
