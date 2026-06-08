# 02 - 扩展 API 完整参考

> 对应设计文档：`docs/design/extensions/01-extension-api.md`

## JavaScript API

```js
class MyExtension {
  // 扩展信息 (必需)
  static get info() {
    return {
      id: 'my-extension',
      name: 'My Extension',
      version: '1.0.0',
      description: 'My custom blocks',
      author: 'username'
    };
  }

  // 积木定义 (必需)
  static getBlocks() {
    return [
      {
        opcode: 'myBlock',
        blockType: 'command',
        text: 'do something with [PARAM]',
        arguments: {
          PARAM: { type: 'string', defaultValue: 'hello' }
        }
      }
    ];
  }

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

Rust 扩展实现 `SfExtension` trait，编译为 Wasm 模块：

```rust
struct MyExtension;
impl SfExtension for MyExtension {
    fn info(&self) -> ExtensionInfo { ... }
    fn blocks(&self) -> Vec<BlockDefinition> { ... }
    fn execute(&self, opcode: &str, args: &[Value], state: &mut RuntimeState) -> Option<Value> { ... }
    fn settings(&self) -> Vec<SettingDefinition> { ... }
    fn on_install(&self) { ... }
    fn on_uninstall(&self) { ... }
}
```

## C/C++ API

C/C++ 扩展通过 Emscripten 编译为 Wasm，导出 C ABI 函数，由 Rust 侧通过 `extern "C"` 调用。
