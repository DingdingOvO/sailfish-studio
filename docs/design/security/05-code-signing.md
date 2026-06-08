# 05 Code Signing

05 - 代码签名

1. 概述

代码签名确保扩展、插件和桌面应用的来源可验证，未被篡改。

2. 扩展/插件签名

· 提交到官方市场的扩展/插件由 Sailfish Studio 官方签名
· 签名密钥存储在 CI/CD Secrets 中，仅构建流程可访问
· 编辑器加载扩展时验证签名，签名无效或缺失时拒绝加载

3. 桌面应用签名

· macOS: Developer ID 证书 + 公证
· Windows: EV Code Signing Certificate
· Linux: GPG 签名

4. 构建来源证明 (SLSA Level 2+)

· GitHub Actions 生成构建来源证明 (actions/attest-build-provenance)
· 用户可验证发布包是否由官方 CI 构建，未被篡改
· 构建配置和依赖锁定文件在 Git 中可审计
