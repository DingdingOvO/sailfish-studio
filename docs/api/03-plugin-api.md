# 03 - 插件 API 完整参考

## 注册

```js
sailfish.plugins.register({
  id: 'my-plugin',
  name: 'My Plugin',
  version: '1.0.0',
  permissions: ['ui', 'theme'],
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

```js
sailfish.theme.register({
  name: 'My Theme',
  colors: { primary: '#FF0000', ... },
  fonts: { sans: '...', mono: '...' }
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
sailfish.hooks.onBeforeSave((project) => { ... });
sailfish.hooks.onAfterCompile((jsCode) => { ... });
```
