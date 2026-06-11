# 02 - 核心概念

[上一章](./01-overview.md) | [下一章：技术架构](./03-architecture.md)

## 2.1 积木
主要的编程方式，操作与 Scratch 相同。底层引擎 Rust 自研，不依赖 Blockly。内置全部 Scratch 3.0 积木及 TurboWarp 增强。扩展可增加新积木。

## 2.2 .sfl 纯文本语言
从积木分叉出的纯文本形态。语法与积木一一对应。文件后缀 `.sfl`。可被 SF Runtime 执行，也可 AOT 编译。提供 VS Code 扩展。

.sfl 的类型系统对标 Rust，支持：基本类型（`any`、`string`、`number`、`float`、`long`、`bigint`、`bool`）、常量（`const`）、结构体、枚举、可选类型（`T?`）、类型别名（`type`）、泛型（`列表<T>`）、联合类型（`T | U`）。

## 2.3 .sf 项目文件
项目保存格式，SQLite 数据库。比 ZIP 紧凑，支持增量写入和实时保存。包含：元信息、校验码、积木树、资源 BLOB、项目设置、扩展列表与设置。

## 2.4 SF 程序包
.sfl 文件和资源打包的可执行单元，后缀 `.sfp`。非加密，可混淆保护。

## 2.5 SF Runtime
命令行工具，执行 .sfl 和 .sfp。核心命令：`sf run`、`sf pack`、`sf new`、`sf check`。有头/无头两种模式。

## 2.6 AOT 编译
将项目编译为原生可执行文件，不含解释器。支持 XML 配置或命令行参数。

## 2.7 扩展
增强语言能力的模块。兼容 TurboWarp JS 扩展，支持 Rust/C++/TS。官方市场 + 社区渠道。

## 2.8 编辑器插件
增强编辑器功能的模块，不增加积木。
