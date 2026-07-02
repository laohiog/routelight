# RouteLight v0.1.0-rc1 Release Notes

Release candidate: v0.1.0-rc1

## 功能摘要

- Windows 优先的轻量级 Tauri 托盘应用。
- 周期性检测公网出口 IPv4 / IPv6、国家、城市、ASN 和 ISP。
- 检测 ChatGPT、OpenAI API、Claude 和 Anthropic API 的 HTTPS 连通性与响应状态。
- 只读读取 Windows 系统代理设置，并枚举疑似 TUN / VPN / 虚拟网卡。
- 显示 DNS 服务器、默认网关、最近 20 次出口 IPv4 变化历史。
- 支持 Copy Diagnostics，将当前诊断快照写入剪贴板。
- 托盘图标按状态显示绿色、黄色、红色或灰色。
- 支持用户主动开启或关闭“随系统启动”。

## 安全边界

- 不修改系统代理。
- 不修改路由表。
- 不切换 VPN 节点。
- 不抓包。
- 不读取浏览器 Cookie。
- 不读取剪贴板。
- 仅在用户点击 Copy Diagnostics 时写入剪贴板。
- 不上传诊断报告。
- 不把诊断报告写入磁盘。
- 不创建 Windows 计划任务。
- 默认不注册开机自启动项；仅当用户主动开启“随系统启动”时，RouteLight 才会通过 Tauri 官方 autostart 插件写入系统自启动配置；用户可随时关闭。
- 不使用 PowerShell / cmd / shell 作为运行时逻辑。
- 不提供 IP 纯净度评分。
- 不提供代理节点、代理订阅或翻墙功能。

## 已知限制

- v0.1.0-rc1 默认每 60 秒后台检测一次，不是毫秒级实时监控。
- IP 地理位置依赖第三方公共 Geo-IP 服务，可能因数据库延迟、CDN 调度或临时网络抖动产生偏差。
- IPv6、DNS、默认网关和虚拟网卡信息是只读诊断线索，不等同于完整网络审计。
- 通知权限关闭或不可用时，提示会降级写入主界面的 Debug Info，不会阻断检测流程。
- 当前测试重点是 Windows 桌面环境；其他操作系统不作为 v0.1.0-rc1 推荐目标。

## 打包与分发

- v0.1.0-rc1 只推荐分发 NSIS 安装包。
- 默认构建命令 `npm run tauri build` 生成 NSIS 安装包。
- 推荐安装包路径：`src-tauri/target/release/bundle/nsis/RouteLight_0.1.0_x64-setup.exe`
- 免安装可执行文件路径：`src-tauri/target/release/routelight.exe`
- MSI / WiX 暂不作为 v0.1.0 发布产物；后续如需 MSI，应单独修复并验证 WiX 打包流程后再启用。

## SHA256

SHA256 checksums should be generated from the final release artifacts and published alongside the GitHub Release assets.

For local builds, generate checksums with:

```powershell
Get-FileHash .\src-tauri\target\release\bundle\nsis\RouteLight_0.1.0_x64-setup.exe -Algorithm SHA256
Get-FileHash .\src-tauri\target\release\routelight.exe -Algorithm SHA256
```

Do not treat checksums in source documentation as canonical if the binaries are rebuilt.

## 人工验收重点

- 验证系统托盘图标出现，并按 Normal / Warning / Error / Unknown 显示正确颜色。
- 验证系统通知在状态变化、出口 IP 变化、恢复 Normal 时触发，且同状态不会重复轰炸。
- 验证断开代理后下一轮刷新能进入风险或错误状态，恢复代理后下一轮刷新能恢复。
- 验证 Copy Diagnostics 内容与当前面板快照一致。
- 验证 Exit 后进程无残留。
