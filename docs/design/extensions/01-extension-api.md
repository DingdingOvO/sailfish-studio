# 01 Extension Api

01 - 扩展 API 参考

1. 概述

Sailfish Studio 扩展 API 允许开发者创建新的积木，扩展 SF 语言的功能。扩展可用 JavaScript、Rust 或 C++ 编写。

> 对应 API 参考文档：`docs/api/02-extension-api.md`

2. JavaScript API

```js
class MyExtension {
  static get info() {
    return {
      id: 'my-extension',
      name: 'My Extension',
      version: '1.0.0',
      description: 'My custom blocks',
      author: 'username'
    };
  }

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

  static execute(opcode, args, runtime) {
    if (opcode === 'myBlock') {
      console.log(args.PARAM);
    }
  }

  // 设置定义 (可选)
  static getSettings() { return [ { key, type, default, title } ]; }

  // 生命周期 (可选)
  static onInstall() { }
  static onUninstall() { }
}

Scratch.extensions.register(new MyExtension());
```

3. Rust API

Rust 扩展实现 SfExtension trait，编译为 Wasm 模块：

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

4. C/C++ API

C/C++ 扩展通过 Emscripten 编译为 Wasm，导出 C ABI 函数，由 Rust 侧通过 extern "C" 调用。
