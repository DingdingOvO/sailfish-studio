# 01 - sf-vm WASM 导出接口

## 概述

sf-vm 编译为 WebAssembly 后，导出以下函数供 JavaScript 调用。

## 函数列表

### sf_vm_create() -> u32

创建新的 RuntimeState，返回指针句柄。

### sf_vm_execute(handle: u32, opcode: *const u8, opcode_len: u32, args_json: *const u8, args_len: u32) -> u64

执行积木操作。opcode 为 UTF-8 字符串，args_json 为 JSON 格式参数。返回值高 32 位为错误码 (0 成功)，低 32 位为结果指针。

### sf_vm_destroy(handle: u32)

销毁 RuntimeState，释放内存。

### sf_vm_compile(handle: u32, project_json: *const u8, json_len: u32) -> u64

编译项目，返回 JS 代码字符串指针。

### sf_vm_get_variable(handle: u32, var_name: *const u8, name_len: u32) -> u64

获取变量值。

### sf_vm_set_variable(handle: u32, var_name: *const u8, name_len: u32, value_json: *const u8, value_len: u32) -> u32

设置变量值，返回 0 成功。
