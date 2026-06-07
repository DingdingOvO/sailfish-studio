# 00 - 文档约定与常量索引

[返回 README](./README.md) | [下一章：项目概述](./01-overview.md)

## 术语约定
- **必须 / MUST**：绝对要求
- **不得 / MUST NOT**：绝对禁止
- **应该 / SHOULD**：强烈推荐
- **不应该 / SHOULD NOT**：不推荐
- **可以 / MAY**：可选

## 魔法数字禁止
代码中不得出现裸数字。所有数值必须定义为具名常量，此处为参考值。

## 常量索引

### 编辑器交互
| 常量名 | 值 | 单位 | 说明 |
|--------|----|------|------|
| MAGNETIC_SNAP_DISTANCE_PX | 8 | px | 积木磁吸距离 |
| HISTORY_STACK_MAX_DEPTH | 200 | 条 | 撤销栈深度 |
| TRAIL_FRAME_COUNT | 6 | 帧 | 拖尾帧数 |
| TRAIL_MAX_ALPHA | 0.8 | — | 拖尾最大透明度 |
| TRAIL_MIN_ALPHA | 0.1 | — | 拖尾最小透明度 |
| BLOCK_NEST_INDENT_PX | 24 | px | 嵌套缩进 |
| BLOCK_VERTICAL_GAP_PX | 4 | px | 垂直间距 |

### 内存
| 常量名 | 值 | 单位 | 说明 |
|--------|----|------|------|
| WASM_HEAP_BUDGET_DEFAULT_MB | 256 | MB | Wasm 堆预算 |
| WASM_MEMORY_WARNING_THRESHOLD_MB | 1024 | MB | 内存警告阈值 |
| AUDIO_STREAMING_MIN_DURATION_SEC | 5 | 秒 | 流式播放阈值 |
| AUDIO_SHORT_CACHE_MAX_COUNT | 128 | 个 | 短音频缓存上限 |
| TEXTURE_CACHE_MAX_COUNT | 256 | 个 | 纹理缓存上限 |
| RESOURCE_IMPORT_WARNING_SIZE_MB | 10 | MB | 大文件警告阈值 |
| JPEG_COMPRESSION_QUALITY | 85 | % | 压缩质量 |

### 舞台与渲染
| 常量名 | 值 | 单位 | 说明 |
|--------|----|------|------|
| STAGE_MIN_WIDTH | 240 | px | 最小宽度 |
| STAGE_MIN_HEIGHT | 180 | px | 最小高度 |
| STAGE_DEFAULT_WIDTH | 480 | px | 默认宽度 |
| STAGE_DEFAULT_HEIGHT | 360 | px | 默认高度 |
| STAGE_MAX_WIDTH | 3840 | px | 最大宽度 (4K) |
| STAGE_MAX_HEIGHT | 2160 | px | 最大高度 (4K) |
| DPR_MIN | 1.0 | — | DPR 最小值 |
| DPR_MAX | 4.0 | — | DPR 最大值 |
| TARGET_FPS | 60 | fps | 目标帧率 |
| FPS_FLUCTUATION_MAX | 5 | fps | 最大帧率波动 |
| FPS_MIN_ACCEPTABLE | 55 | fps | 最低可接受帧率 |

### 运行时
| 常量名 | 值 | 单位 | 说明 |
|--------|----|------|------|
| CLONE_DEFAULT_MAX | 300 | 个 | 克隆体默认上限 |
| COMPILE_TIME_TARGET_MS | 200 | ms | 1000 积木编译目标 |
| MP4_DEFAULT_FPS | 30 | fps | MP4 默认帧率 |
| MP4_HIGH_FPS | 60 | fps | MP4 高帧率 |
| GIF_MAX_DURATION_SEC | 30 | 秒 | GIF 最大时长 |
| GIF_DEFAULT_FPS | 15 | fps | GIF 默认帧率 |

### 协作
| 常量名 | 值 | 单位 | 说明 |
|--------|----|------|------|
| COLLAB_MAX_PARTICIPANTS | 32 | 人 | 单房间最大人数 |
| COLLAB_LATENCY_MAX_MS | 200 | ms | 最大协作延迟 |
| OFFLINE_LOG_MAX_HOURS | 72 | 小时 | 离线日志保留时长 |

### 日志
| 常量名 | 值 | 单位 | 说明 |
|--------|----|------|------|
| LOG_LEVEL_TRACE | 0 | — | 追踪 |
| LOG_LEVEL_DEBUG | 1 | — | 调试 |
| LOG_LEVEL_INFO | 2 | — | 信息 |
| LOG_LEVEL_WARN | 3 | — | 警告 |
| LOG_LEVEL_ERROR | 4 | — | 错误 |
| LOG_LEVEL_FATAL | 5 | — | 致命 |
| LOG_BUFFER_SIZE | 10000 | 条 | 缓冲区大小 |
| LOG_CONSOLE_BUFFER_SIZE | 50000 | 条 | 控制台缓冲区 |
| LOG_BUFFER_MEMORY_MAX_MB | 16 | MB | 日志最大内存 |
| LOG_MAX_WRITES_PER_SEC_NORMAL | 100 | 条/秒 | 正常模式写入 |
| LOG_MAX_WRITES_PER_SEC_DEBUG | 10000 | 条/秒 | 调试模式写入 |
