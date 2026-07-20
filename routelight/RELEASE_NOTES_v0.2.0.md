# RouteLight v0.2.0 Release Notes

发布日期：2026-07-20

## 主要更新

- 新增 Google AI 匿名可用性检测，通过 `google.com/ai` 的最终跳转地址和明确的地区限制页面判断当前网络出口是否可进入 AI Mode。
- Google AI 结果细分为“可用”“地区不支持”“需人工确认”“不可达”和“无法判定”，减少仅凭 HTTP 状态码造成的误判。
- 移除 OpenAI API 与 Anthropic API 的请求、状态判断、模拟数据和界面展示，保留 ChatGPT 与 Claude 的 HTTPS 连通性检测。
- 更新诊断文本、状态汇总、通知逻辑和技术文档，使其与新的三项检测保持一致。
- 新增 Google AI 响应分类与状态序列化测试，并加入 GitHub CI 检查。

## 隐私与安全边界

- Google AI 检测不登录 Google 账号，不读取浏览器 Cookie、账号信息或凭证。
- RouteLight 仍然只做只读诊断，不修改系统代理或路由表，不抓包，也不提供代理或 VPN 功能。
- 账号类型、语言、实验分组、设备位置或验证码可能使浏览器实际结果与匿名检测不同；遇到不确定响应时会提示人工确认。

## 发布产物

- Windows NSIS 安装包：`RouteLight_0.2.0_x64-setup.exe`
- SHA256：`ABE2126EB4B3DEF9FD9AF6E0B2756E263115C9D158864C70CCA1AF9BEF09542F`

## 已知限制

- 当前推荐和验证的平台是 Windows，默认只发布 NSIS 安装包。
- 公共 Geo-IP 数据、目标服务 CDN、验证码和临时网络状态都可能影响单次检测结果。
- 后台检测默认每 60 秒刷新一次，不是毫秒级实时监控。
