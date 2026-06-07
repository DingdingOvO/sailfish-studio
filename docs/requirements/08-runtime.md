# 08 - 运行时与打包

[上一章](./07-extensions.md) | [下一章：跨平台](./09-platform.md)

## 8.1 SF Runtime
命令行工具。`sf run` 执行 .sfl 或 .sfp。有头模式（图形）和无头模式（服务器后端）。打包可选内嵌或依赖外部 Runtime。

## 8.2 AOT 编译
XML 配置或命令行参数。编译流程：解析→优化→LLVM 生成机器码→链接启动代码→输出原生可执行文件。

## 8.3 混淆
`sf-obfuscator` 保护商业代码。不可逆。
