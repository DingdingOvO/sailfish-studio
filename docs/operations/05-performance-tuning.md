# 05 - 性能调优

> 对应设计文档：`docs/design/testing/02-performance-bench.md`
> 对应需求文档：`docs/requirements/11-nfr.md`

## Wasm 内存优化

- 使用 `wasm-opt -Os` 优化 WASM 文件大小和运行时性能
- 使用 Rust 默认分配器（已优于 `wee_alloc`，后者已停止维护且存在内存泄漏问题）
- 资源懒加载，避免全量加载到 Wasm 内存
- 使用 `console_error_panic_hook` 仅在 Debug 构建中启用，Release 构建中移除以减小体积
- 启用 LTO（Link-Time Optimization）：在 `Cargo.toml` 中设置 `[profile.release] lto = true`

```toml
[profile.release]
lto = true
opt-level = "s"      # 优化体积
codegen-units = 1    # 单编译单元，更好的优化
```

## 渲染性能

- 使用 WebGL2 批量绘制（按纹理排序，减少状态切换）
- 脏矩形检测避免全画布重绘
- 克隆体使用对象池，避免频繁分配/释放
- SVG 路径预三角化并缓存顶点缓冲区
- 纹理缓存上限 `TEXTURE_CACHE_MAX_COUNT` (256)，超出生效 LRU 淘汰

## CDN 缓存策略

| 资源类型 | Cache-Control | 说明 |
|---------|---------------|------|
| 静态资源 (JS/CSS/WASM/字体) | `max-age=31536000, immutable` | 内容哈希文件名，可永久缓存 |
| HTML 文件 | `max-age=0, must-revalidate` | 每次验证是否有新版本 |
| API 响应 | `max-age=60` | 1 分钟短缓存 |
| 图片/音频资源 | `max-age=86400` | 24 小时缓存 |

## 数据库查询优化

- 积木树查询使用预编译语句（Prepared Statements），避免重复解析 SQL
- 资源 BLOB 仅在需要时读取，不在项目加载时全量读入内存
- 定期 `VACUUM` 清理 SQLite 碎片空间
- 为高频查询字段创建索引（如 `blocks.opcode`、`targets.name`）
- 使用 WAL 模式提升并发读写性能：`PRAGMA journal_mode=WAL`

## 编辑器性能

- 积木拖拽帧率波动 ≤ 3fps（使用 `requestAnimationFrame` 同步渲染）
- 工具箱搜索过滤延迟 < 100ms（使用防抖 debounce 300ms）
- 项目打开时间（1000 积木）< 1 秒（增量加载，先显示框架后填充内容）
- 撤销/重做栈深度限制 `HISTORY_STACK_MAX_DEPTH` (200)，超出生效丢弃最旧记录

## 协作性能

- 操作延迟 < `COLLAB_LATENCY_MAX_MS` (200ms)
- 32 人同时编辑帧率影响 < 5fps
- 光标位置广播节流 100ms，避免高频消息
- 操作日志批量传输，减少 WebSocket 消息数量
