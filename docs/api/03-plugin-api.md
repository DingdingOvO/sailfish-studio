# 03 - 插件 API 完整参考

> 对应设计文档：`docs/design/extensions/03-plugin-api.md`

## 注册

```js
sailfish.plugins.register({
  id: 'my-plugin',
  name: 'My Plugin',
  version: '1.0.0',
  permissions: ['ui', 'theme', 'shortcuts'],
  activate() { /* 插件激活 */ },
  deactivate() { /* 插件停用 */ }
});
```

## UI 扩展

```js
// 添加面板
sailfish.ui.registerPanel({ id, title, icon, render, position: 'right' });

// 添加工具栏按钮
sailfish.ui.addToolbarButton({ id, icon, tooltip, onClick, position: 'left' });

// 注册右键菜单
sailfish.ui.registerContextMenu({ id, label, icon, predicate, action });
```

## 主题扩展

主题可覆盖颜色、字体、圆角、阴影：

```js
sailfish.theme.register({
  name: 'My Theme',
  colors: { primary: '#FF0000', ... },
  fonts: { sans: '...', mono: '...' },
  radii: { sm: 4, md: 6, lg: 8 },
  shadows: { card: '0 2px 8px rgba(0,0,0,0.3)' }
});
```

## 快捷键

```js
sailfish.shortcuts.register('my-command', {
  key: 'Ctrl+Shift+M',
  callback: () => { ... }
});
```

## 编辑器钩子

```js
sailfish.hooks.onProjectOpen((project) => { ... });
sailfish.hooks.onProjectClose((project) => { ... });
sailfish.hooks.onBeforeSave((project) => { ... });
sailfish.hooks.onAfterSave((project) => { ... });
sailfish.hooks.onBeforeCompile((project) => { ... });
sailfish.hooks.onAfterCompile((jsCode) => { ... });
```
