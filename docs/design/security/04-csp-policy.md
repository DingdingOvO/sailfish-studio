# 04 Csp Policy

04 - CSP 策略

1. 概述

Content Security Policy (CSP) 限制浏览器中可执行的脚本和可加载的资源。

2. Web 端 CSP 头

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'wasm-unsafe-eval';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: blob:;
  connect-src 'self' https://api.sailfish.studio wss://collab.sailfish.studio;
  worker-src 'self' blob:;
  font-src 'self';
```

3. 沙箱 iframe (社区渠道扩展)

```
Content-Security-Policy:
  default-src 'none';
  script-src 'self';
  connect-src 'none';
```

4. 桌面端

· 桌面端不使用 CSP 头，但通过操作系统级沙箱实现等效限制
