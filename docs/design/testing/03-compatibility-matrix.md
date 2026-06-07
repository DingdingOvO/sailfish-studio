# 03 Compatibility Matrix

03 - 兼容性矩阵

1. 浏览器兼容性

浏览器 最低版本 说明
Chrome 90+ 完整支持
Firefox 90+ 完整支持
Safari 15+ WebGL2 部分特性降级
Edge 90+ 等同于 Chrome

2. 操作系统兼容性

系统 最低版本 说明
Windows 10 (21H2+) 桌面端 + Web 端
macOS 11 (Big Sur+) 桌面端 (Apple Silicon 原生) + Web 端
Linux Ubuntu 20.04+ 桌面端 + Web 端
Android 10+ 平板端 (Chrome 或 WebView)

3. CI 测试矩阵 (GitHub Actions)

· OS: ubuntu-latest, macos-latest, windows-latest
· Rust: stable, beta
· Node: 24 LTS
· Browser: Chromium, Firefox, WebKit (Playwright)
