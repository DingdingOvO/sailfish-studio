# 02 - 监控与告警

## 错误追踪 (Sentry)

- 集成 Sentry SDK 到编辑器和扩展进程
- 告警规则: 5 分钟内 10 个以上错误触发通知

## 性能监控 (Cloudflare Analytics)

- Web 端: 页面加载时间、Web Vitals (LCP、FID、CLS)
- API 端: 请求延迟、错误率、QPS

## 自定义指标

- 协作房间数、活跃用户数
- 扩展安装量
- Wasm 内存使用趋势

## 告警渠道

- 严重事件 (P0/P1): Slack + 邮件 + 电话
- 一般事件 (P2/P3): Slack 通知
