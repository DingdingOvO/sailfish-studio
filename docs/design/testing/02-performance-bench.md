# 02 Performance Bench

02 - 性能基准

1. 概述

定义 Sailfish Studio 必须满足的性能基准。

2. 编译器基准

· 1000 积木项目编译时间 < 200ms
· 增量编译 (修改 1 个积木) < 20ms
· 测试项目: 标准测试集 bench/1000-blocks.sf

3. 渲染器基准

· 1000 个角色同时移动: ≥ 60fps
· 10000 个克隆体: ≥ 30fps
· 舞台缩放 (25% → 400%): 缩放过程中 ≥ 55fps
· 测试设备: 中等配置 (Intel i5 / 8GB RAM / 集成显卡)

4. 编辑器基准

· 积木拖拽帧率波动 ≤ 3fps
· 工具箱搜索过滤延迟 < 100ms
· 项目打开时间 (1000 积木) < 1 秒

5. 协作基准

· 操作延迟 < 200ms (正常网络)
· 32 人同时编辑帧率影响 < 5fps

6. 测试工具

· Rust: criterion + cargo bench
· Web: Lighthouse CI + Playwright 性能录制
