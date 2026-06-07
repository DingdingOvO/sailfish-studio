# 02 - 扩展 API 完整参考

## JavaScript API

```js
class MyExtension {
  // 扩展信息 (必需)
  static get info() { return { id, name, version, description, author }; }

  // 积木定义 (必需)
  static getBlocks() { return [ { opcode, blockType, text, arguments } ]; }

  // 积木执行 (必需)
  static execute(opcode, args, runtime) { /* 返回 Value 或 Promise<Value> */ }

  // 设置定义 (可选)
  static getSettings() { return [ { key, type, default, title } ]; }

  // 生命周期 (可选)
  static onInstall() { }
  static onUninstall() { }
}

Scratch.extensions.register(new MyExtension());
```

## Rust API

```rust
impl SfExtension for MyExtension {
    fn info(&self) -> ExtensionInfo { ... }
    fn blocks(&self) -> Vec<BlockDefinition> { ... }
    fn execute(&self, opcode: &str, args: &[Value], state: &mut RuntimeState) -> Option<Value> { ... }
    fn settings(&self) -> Vec<SettingDefinition> { ... }
    fn on_install(&self) { ... }
    fn on_uninstall(&self) { ... }
}
```
