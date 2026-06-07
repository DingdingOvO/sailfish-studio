# 03 - .sfl 语言指南

## 概述

.sfl 是 Sailfish Studio 的纯文本编程语言，与积木一一对应。可以用文本编辑器编写，然后在编辑器中导入为积木，也可以直接通过 SF Runtime 运行。

## 变量与类型

```
var score: number = 0          # 双精度浮点数
let name: string = "Player"    # 字符串
const MAX: number = 100        # 常量
var flag: bool = true          # 布尔值
var items: list<number> = [1, 2, 3]  # 列表
var player: struct { name: string, hp: number } = { name: "Hero", hp: 100 }
```

## 控制流

```
if score > 100:
    broadcast "win"
else:
    broadcast "continue"

repeat 10:
    this.moveSteps(10)

forever:
    this.nextFrame()
```

## 函数

```
func greet(name: string) -> void:
    this.say("Hello, " + name)

func add(a: number, b: number) -> number:
    return a + b
```

## 广播

```
broadcast "game_start"
broadcast "game_over" and wait

on "game_start":
    this.show()
```
