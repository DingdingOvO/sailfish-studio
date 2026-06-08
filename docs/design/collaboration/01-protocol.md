# 01 - 协作协议

[返回 README](../README.md)

## 1. 操作类型

协作系统使用 OT (Operational Transformation) 算法处理并发编辑。定义以下原子操作:

### 1.1 积木操作

- `block_insert(block_id, parent_id, position, block_data)`
- `block_delete(block_id)`
- `block_move(block_id, new_parent_id, new_position)`
- `block_update_field(block_id, field_name, new_value)`

### 1.2 变量操作

- `variable_set(variable_id, new_value)`
- `list_insert(list_id, index, value)`
- `list_delete(list_id, index)`

### 1.3 角色操作

- `sprite_create(sprite_data)`
- `sprite_delete(sprite_id)`
- `sprite_update_property(sprite_id, property, value)`

## 2. 冲突解决

- **积木树**: OT 变换函数保证操作可交换。
- **变量**: 最后写入者获胜 (LWW)。
- **造型**: 编辑期间锁定，其他用户看到锁定状态。
