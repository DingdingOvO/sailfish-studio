# 06 Vscode Extension

06 - VS Code 扩展设计

1. 概述

提供 .sfl 文件的语法高亮、智能补全、错误诊断和代码片段。

2. 功能

2.1 语法高亮

· 基于 TextMate 语法定义，高亮关键字、字符串、数字、注释
· 关键字用蓝色，字符串用橙色，数字用绿色，注释用灰色斜体

2.2 智能补全

· 基于 Language Server Protocol (LSP) 实现
· 补全关键字、变量名、函数名、类型名
· 输入 . 时补全结构体字段或方法
· 输入 " 时补全已知广播消息名

2.3 错误诊断

· 编译时错误在编辑器中以红色波浪线标注
· 悬停显示错误详情
· 错误包括: 类型不匹配、未定义变量、语法错误

2.4 代码片段

· func → 生成函数模板
· if → 生成 if 块模板
· repeat → 生成 repeat 块模板
· struct → 生成结构体模板

3. 发布

· 通过 VS Code Marketplace 发布，ID 为 sailfish-studio.sfl-language
· 版本号跟随 Sailfish Studio 主版本
