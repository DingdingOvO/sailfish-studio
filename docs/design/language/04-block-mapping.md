# 04 Block Mapping

04 - 积木 ↔ .sfl 映射规则

1. 概述

定义 Scratch 积木与 .sfl 语法之间的双向转换规则，确保积木编辑器中的任何操作都能无损转换为 .sfl 代码，反之亦然。

2. 转换规则表

2.1 运动类

· move (steps) steps → this.moveSteps(steps)
· turn right (degrees) degrees → this.turnRight(degrees)
· turn left (degrees) degrees → this.turnLeft(degrees)
· go to x: (x) y: (y) → this.goToXY(x, y)
· glide (secs) secs to x: (x) y: (y) → this.glide(secs, x, y)

2.2 事件类

· when green flag clicked → on "flag_clicked":
· broadcast (message) → broadcast "message"
· broadcast (message) and wait → broadcast "message" and wait
· when I receive (message) → on "message":

2.3 控制类

· wait (seconds) seconds → await this.wait(seconds)
· repeat (count) → repeat count:
· forever → forever:
· if (condition) then → if condition:
· if (condition) then ... else → if condition: ... else:
· wait until (condition) → while not condition:

2.4 变量类

· set [var] to (value) → var = value
· change [var] by (delta) → var += delta
· show variable [var] → this.showVariable("var")
· hide variable [var] → this.hideVariable("var")

2.5 列表类

· add (item) to [list] → list.push(item)
· delete (index) of [list] → list.deleteAt(index)
· insert (item) at (index) of [list] → list.insertAt(index, item)
· item (index) of [list] → list[index]

2.6 自定义积木

· define (name) (params...) → func name(params...):
· (name) (args...) → name(args...)

3. 命名转换规则

· 积木中文标签 → 英文标识符（通过积木 ID 映射，保证跨语言一致）
· 变量名保留原始名称，空格转为下划线
· 列表名保留原始名称，后缀 _list
· 广播消息名保留原始名称，空格转为下划线
