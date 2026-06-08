# 03 Plugin Api

03 - 插件 API 参考

1. 概述

插件 API 允许开发者扩展编辑器的功能（不增加新积木），修改 UI、主题、快捷键等。插件需通过审核才能进入官方市场。

> 对应 API 参考文档：`docs/api/03-plugin-api.md`

2. 插件能力

2.1 UI 扩展

· 添加自定义面板: sailfish.ui.registerPanel(panelConfig)
· 修改工具栏: sailfish.ui.addToolbarButton(buttonConfig)
· 注册右键菜单: sailfish.ui.registerContextMenu(menuConfig)

2.2 主题扩展

· 注册新主题: sailfish.theme.register(themeConfig)
· 主题可覆盖颜色、字体、圆角、阴影

2.3 快捷键

· 注册快捷键: sailfish.shortcuts.register(command, config)
· config 包含 key 和 callback 属性
· 快捷键冲突时提示用户手动解决

2.4 编辑器钩子

· 项目打开/关闭: onProjectOpen(callback), onProjectClose(callback)
· 保存前/后: onBeforeSave(callback), onAfterSave(callback)
· 编译前/后: onBeforeCompile(callback), onAfterCompile(callback)

2.5 生命周期

· activate(): 插件激活时调用
· deactivate(): 插件停用时调用

3. 插件描述文件

```json
{
  "id": "my-plugin",
  "version": "1.0.0",
  "name": "My Plugin",
  "description": "Custom editor plugin",
  "author": "username",
  "permissions": ["ui", "theme", "shortcuts"],
  "entry": "./index.js"
}
```
