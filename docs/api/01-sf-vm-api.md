# 01 - sf-vm WASM 导出接口

## 概述

sf-vm 编译为 WebAssembly 后，通过 `wasm-bindgen` 导出以下函数供 JavaScript 调用。所有字符串参数和返回值由 wasm-bindgen 自动编解组（`&str` / `String`），无需手动传递指针和长度。

> 对应实现代码：`docs/development/09-implementation-checklist.md` Section 3.6

## 函数列表

### sf_vm_create() -> *mut RuntimeState

创建新的 RuntimeState，返回裸指针句柄。JavaScript 侧收到的是一个不透明的数值 ID，无需关心内部类型。创建后 WASM 模块会自动调用 `init()` 函数设置 panic hook。

```js
const state = sf_vm_create();
```

### sf_vm_compile(state_ptr: *mut RuntimeState, project_json: &str) -> String

编译项目。`project_json` 为 JSON 格式的项目数据，返回编译生成的 JavaScript 代码字符串。

```js
const jsCode = sf_vm_compile(state, jsonStr);
```

### sf_vm_execute(state_ptr: *mut RuntimeState, opcode: &str, args_json: &str) -> Result\<String, JsValue\>

执行积木操作。`opcode` 为积木操作码（如 `"motion_move_steps"`），`args_json` 为 JSON 格式参数。成功时返回结果 JSON 字符串，失败时抛出 JsValue 异常。

```js
try {
  const result = sf_vm_execute(state, "motion_move_steps", '{"STEPS": 10}');
} catch (e) {
  console.error("执行失败:", e);
}
```

### sf_vm_get_variable(state_ptr: *mut RuntimeState, var_name: &str) -> Result\<String, JsValue\>

获取变量值，返回 JSON 格式字符串。

```js
const value = sf_vm_get_variable(state, "score");
```

### sf_vm_set_variable(state_ptr: *mut RuntimeState, var_name: &str, value_json: &str) -> Result\<(), JsValue\>

设置变量值。`value_json` 为 JSON 格式的值。成功时无返回值，失败时抛出异常。

```js
sf_vm_set_variable(state, "score", '{"type":"number","value":42}');
```

### sf_vm_destroy(state_ptr: *mut RuntimeState)

销毁 RuntimeState，释放内存。调用后 `state_ptr` 不再有效，禁止继续使用。

```js
sf_vm_destroy(state);
```
