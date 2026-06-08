# 02 - SF 语言语法规范

[返回 README](../README.md)

## 1. 设计原则

- **与积木一一对应**: 每个 Scratch 积木有对应的 .sfl 语法。
- **可读性优先**: 使用缩进定义块，关键字简洁。
- **类型安全**: 编译时检查类型错误。

## 2. 基本语法

### 2.1 变量声明与赋值

```
var score: number = 0
let name: string = "Player"
const MAX_SPEED: number = 100
```

### 2.2 函数定义与调用

```
func moveSteps(steps: number) -> void:
  if steps > 0:
    this.x += steps * cos(this.direction)
    this.y += steps * sin(this.direction)
  else:
    this.x -= abs(steps) * cos(this.direction)
    this.y -= abs(steps) * sin(this.direction)
```

### 2.3 控制流

```
if condition:
  # 代码
else if other_condition:
  # 代码
else:
  # 代码

repeat count:
  # 循环体，count 必须为正整数

forever:
  # 无限循环 (等价于 repeat Infinity)

while condition:
  # 条件循环
```

### 2.4 广播与事件

```
broadcast "game_start"
broadcast "game_start" and wait

on "game_start":
  # 事件处理
```

### 2.5 结构体与枚举

```
struct Player:
  name: string
  score: number
  position: struct { x: number, y: number }

enum Direction:
  Up, Down, Left, Right
```
