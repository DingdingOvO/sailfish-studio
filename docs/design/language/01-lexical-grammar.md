# 01 Lexical Grammar

01 - SF 语言词法规则

1. 概述

SF 语言 (.sfl) 的词法规则定义代码的最小组成单元。

2. Token 类型

2.1 关键字

```
var, let, const, func, return, void,
if, else, elif, repeat, forever, while,
broadcast, and, wait, on,
struct, enum, type, import, export
```

2.2 标识符

· 规则: [a-zA-Z_][a-zA-Z0-9_]*
· 变量名、函数名、类型名均使用标识符
· 区分大小写

2.3 字面量

· 数字: [0-9]+(\.[0-9]+)?（整数和浮点数统一解析）
· 字符串: "..." 双引号包裹，支持 \n、\t、\"、\\ 转义
· 布尔: true、false
· 空值: null

2.4 运算符

· 算术: +, -, *, /, %
· 比较: ==, !=, >, <, >=, <=
· 逻辑: and, or, not
· 赋值: =, +=, -=, *=, /=
· 成员访问: .
· 箭头: ->

2.5 注释

· 单行: # 到行末
· 多行: ### 开始，### 结束（可嵌套）
