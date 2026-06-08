# 04 Fuzzing Strategy

04 - 模糊测试策略

1. 概述

对核心模块进行模糊测试，发现边界情况和安全漏洞。

2. 测试目标

· 项目加载器 (sf-parser): 随机字节输入，验证不会 panic，错误处理正确
· 编译器 (sf-vm-compiler): 随机 AST 输入，验证生成的 JS 代码合法
· 协作协议 (collaboration): 随机操作序列，验证 OT 变换结果一致

3. 工具

· Rust: cargo-fuzz (libFuzzer)
· 持续运行时间: 每次 PR ≥ 1 小时，发布前 ≥ 24 小时

4. CI 集成

· 每次 push 到 main 时运行模糊测试 1 小时
· 发布候选版本时运行模糊测试 24 小时，无崩溃方可发布
· 发现崩溃时自动生成最小复现用例并归档
