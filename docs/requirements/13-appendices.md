# 13 - 附录

[上一章](./12-roadmap.md) | [返回 README](./README.md)

## 术语表
| 术语 | 描述 |
|------|------|
| .sf | Sailfish 项目文件，SQLite 格式 |
| .sfl | Sailfish 纯文本语言 |
| .sfp | SF 程序包 |
| .sprite | 角色数据文件 |
| .backdrop | 背景数据文件 |
| SF Runtime | 独立命令行运行时 |
| AOT 编译 | 预先编译为原生机器码 |

## 支持语言 (50种)
zh-CN, zh-TW, en, ja, ko, fr, de, es, pt-BR, pt, it, ru, ar, hi, bn, id, ms, th, vi, tr, nl, pl, uk, cs, sv, da, fi, nb, hu, ro, el, he, fa, ur, ta, te, mr, gu, kn, ml, pa, sw, fil, sr, hr, sk, bg, lt, lv, et, sl, is, ca, eu, gl, cy, eo

## 仓库总览
| 仓库 | 包数 | 说明 |
|------|-----|------|
| sf-core | 6 核心 crates（总计 28 包） | 核心引擎 |
| sf-editor | 13 | 编辑器 |
| sf-tools | 8 | 工具 |
| sf-services | 6 | 云端 |
| sf-extensions | 21+ | 扩展 |
| sf-docs | — | 文档 (VitePress) |
| sf-runtime | — | 独立运行时 (Rust CLI) |
| sf-aot-compiler | — | AOT 编译器 (Rust + LLVM) |

## 首批高级扩展 (21个)
舞台分辨率与 DPR、Pen Plus、HTML 编码、文本工具、克隆增强、事件增强、外观增强、大整数运算、位运算、进制转换、手柄、iframe 嵌入、视频链接、音频链接、Fetch、Files、本地存储、Three.js 3D、Babylon.js 3D、JSON 数据、XML 数据
