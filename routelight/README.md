# RouteLight

RouteLight 是一款专为 AI 开发者、网络工程人员和桌面工具用户设计的轻量级 **网络连通性状态面板与路由可达性检测工具**。它能够常驻 Windows 系统托盘，后台周期性检测本机公网出口位置、系统代理状态、虚拟网卡状态以及关键 AI 服务的连通性，帮助用户识别当前网络出口与预期配置是否一致。

---

## 📌 项目简介

RouteLight 旨在以轻量、只读、低打扰的方式，为用户提供周期性更新的网络环境视图。它通过后台轻量端点握手，在系统托盘图标颜色中直观呈现当前网络连通性与出口状态：

* 绿色：Normal，当前检测结果符合预期
* 黄色：Warning，存在潜在风险或部分检测异常
* 红色：Error，出口位置或关键服务连通性存在明显异常
* 灰色：Unknown，当前状态无法可靠判断

> ⚠️ **重要声明 (It is NOT)**
>
> * **它不是 VPN / 代理客户端**：RouteLight 自身不提供任何网络代理协议、节点加密、流量转发或网络接入能力。
> * **它不是安全审计工具**：不提供任何 IP 纯净度、欺诈分值（IP Fraud Score）或安全信誉评分。
> * **它不提供、不引导、不协助建立代理、VPN、专线或其它网络接入通道**：RouteLight 仅用于本机网络状态与服务连通性检测。用户应遵守所在地法律法规，并使用依法合规的网络接入服务。
> * **它不保证任何第三方服务可访问**：RouteLight 只展示检测结果，不承诺任何 AI 服务、网站或 API 在特定网络环境下可用。

---

## 🛠️ 功能列表

1. **公网出口探测**
   周期性获取公网出口 IPv4 / IPv6，并映射国家、城市、ISP 与 ASN（自治系统）。

2. **AI 服务连通性检测**
   周期性检测 `ChatGPT`、`OpenAI API`、`Claude` 和 `Anthropic API` 的端点连接延迟与 HTTP 状态。

3. **本机代理设置感知**
   只读检索 Windows 系统代理设置，包括 `ProxyEnable`、`ProxyServer` 和 `AutoConfigURL`。RouteLight 不会修改任何系统代理配置。

4. **虚拟网卡适配器识别**
   扫描本机活跃网卡，识别疑似 TUN / TAP / VPN 虚拟适配器，并列出原始 DNS 服务器与默认网关。该信息仅用于本机网络拓扑提示，不创建、不修改、不启用任何 VPN、TUN/TAP 或代理配置。

5. **出口 IP 变动历史**
   在内存中保留最近 20 次 IPv4 出口变化记录，便于判断当前出口是否发生过切换。

6. **防轰炸系统通知**
   默认每 60 秒在后台自动检测一次。状态变动或 IP 出口发生变化时，会在下一轮检测完成后触发桌面通知；相同状态下不会重复弹窗轰炸。

7. **无边框极简面板**
   支持标题栏拖拽、折叠式 Debug 信息栏、一键复制快照诊断。

8. **可选随系统启动**
   默认不注册开机自启动项。仅当用户主动开启“随系统启动”时，RouteLight 才会通过 Tauri 官方 autostart 插件写入系统自启动配置；用户可随时关闭。

9. **防并发保护**
   在手动刷新、后台自动刷新和托盘刷新过程中使用并发保护，避免重复网络探测导致状态不同步。

---

## 🔒 安全边界与隐私保护 (Strict Non-intrusiveness)

RouteLight 严格遵循只读、非侵入、低权限原则：

* 🚫 **不修改代理**
  不主动开启、关闭或修改 Windows 系统代理设置，不修改 PAC 脚本。

* 🚫 **不干预路由**
  不主动修改路由表，不切换 VPN 节点，不切换代理软件。

* 🚫 **不创建网络通道**
  不提供代理协议、VPN、隧道、节点、订阅转换或任何网络接入能力。

* 🚫 **不劫持流量**
  不进行任何本地网络抓包，不读取浏览器 Cookie，不读取敏感凭据。

* 🚫 **不嗅探剪贴板**
  内置复制诊断功能只会在用户主动点击时写入剪贴板；程序不会读取、监听或监视剪贴板内容。

* 🚫 **不上传诊断报告**
  RouteLight 会向固定公网检测端点发起连通性探测请求，例如出口 IP、Geo-IP 和 AI 服务端点检测；但不会上传诊断报告、代理配置、剪贴板内容、本地文件或用户自定义数据。

* 🚫 **不把诊断报告写入磁盘**
  诊断信息仅在内存中生成，并在用户主动点击复制时写入剪贴板。运行时不会将诊断报告写入本地磁盘。

* 🚫 **不滥用命令**
  程序运行时不执行 `cmd`、`PowerShell` 或外部 Shell 脚本命令。

* ✅ **自启动需用户主动选择**
  默认不注册开机自启动项。仅当用户主动开启“随系统启动”时，RouteLight 才会通过 Tauri 官方 autostart 插件写入系统自启动配置；用户可随时关闭。RouteLight 不创建 Windows 计划任务（Task Scheduler），不手写注册表，不使用 PowerShell / cmd / shell 实现自启动，退出后不残留后台进程。安装程序所创建的正常程序文件、桌面快捷方式和卸载信息除外。

---

## ⚖️ 合规使用声明

RouteLight 是一个本机网络状态与服务连通性检测工具。它不提供、不引导、不协助建立任何代理、VPN、专线、隧道或其它网络接入通道。

用户应自行确保其网络环境、访问行为和使用场景符合所在地法律法规、组织内部 IT 政策和第三方服务条款。RouteLight 仅展示检测结果，不对用户网络接入方式、第三方服务可访问性或合规性作出保证。

---

## ⚙️ 开发与运行环境

### 1. 前置准备 (Prerequisites)

* **操作系统**：Windows 10 / Windows 11，且装有 [WebView2 运行时](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)（Windows 11 通常已默认内置）。
* **Node.js**：LTS 版本，推荐 Node 18+。
* **Rust 构建工具**：安装 [Rustup](https://rustup.rs/) 及 Microsoft C++ Build Tools（VS 2019 / 2022 开发人员命令行）。

### 2. 运行方式 (Development)

在 `routelight` 目录下运行：

```bash
# 安装前端依赖
npm install

# 启动 Tauri 开发服务器（真实网络探测模式）
npm run tauri dev
```

### 3. Mock 模式调试 (Mock Mode)

可以通过注入环境变量 `ROUTELIGHT_MOCK_STATUS` 快速测试前端面板与指示灯在各种网络状态下的渲染反应：

```powershell
# Windows PowerShell 下的测试命令
$env:ROUTELIGHT_MOCK_STATUS="normal"; npm run tauri dev   # 绿灯 Normal 状态
$env:ROUTELIGHT_MOCK_STATUS="warning"; npm run tauri dev  # 黄灯 Warning 状态
$env:ROUTELIGHT_MOCK_STATUS="error"; npm run tauri dev    # 红灯 Error 状态
```

Windows CMD 环境可使用 `npm.cmd run tauri dev`，并使用相应的 CMD 环境变量设置方式：

```cmd
set ROUTELIGHT_MOCK_STATUS=normal && npm.cmd run tauri dev
```

---

## 📦 打包构建 (Packaging)

若要生成 Windows 平台的免安装可执行文件（`.exe`）与推荐安装包（NSIS），请在 `routelight` 目录下运行：

```bash
npm run tauri build
```

编译产物将被输出在：

* 免安装可执行文件：`src-tauri/target/release/routelight.exe`
* NSIS 安装包：`src-tauri/target/release/bundle/nsis/RouteLight_0.1.0_x64-setup.exe`
  或匹配通配符：`RouteLight_*_x64-setup.exe`

v0.1.0 默认仅构建并推荐分发 NSIS 安装包。MSI / WiX 打包链路暂不作为 v0.1.0 推荐分发产物；如后续需要 MSI，应单独修复并验证 WiX `light.exe` 打包流程后再启用。

当前前端是无 bundler 的静态 HTML / CSS / JS，并通过 Tauri 注入的 `window.__TAURI__` 调用 `invoke`、`listen` 与窗口拖拽 API，因此 v0.1.0 暂时保留 `withGlobalTauri`。同时，Tauri 配置已启用最小 CSP，限制脚本、样式、图片和 IPC 连接来源。

---

## 🔔 通知权限说明

RouteLight 出口与状态提示依赖 Windows 原生操作中心通知系统：

* 可以在 Windows 的 **设置 -> 系统 -> 通知** 中为本应用开启或关闭通知。
* 如果用户未授予通知权限或通知功能不可用，应用会自动、安全地将提示回退写入主界面的 Debug Info 控制台日志中，不会导致程序异常终止或崩溃。

---

## ⚠️ 已知限制 (Known Limitations)

* **地理位置解析误差**
  IP 地址所映射的国家、城市、ASN 及运营商等信息依赖第三方公共 Geo-IP 接口，例如 ipwho.is，并在部分情况下 fallback 到 ipinfo.io。由于公共数据库更新延迟、CDN 调度和运营商路由变化，结果可能存在误差。

* **端点检测受限性**
  AI 端点检测是对目标域名进行实际 HTTPS 握手测试。部分接口可能受限于本地网络服务商的临时解析状况、目标服务的 CDN 调度、网关策略或速率限制。

* **非毫秒级实时监控**
  后台自动检测通过常驻线程以默认 60 秒的间隔进行周期刷新，非毫秒级连续高频监听。

* **DNS / 网关原始性**
  DNS 和网关列表为从系统 IP Helper 中获取的原始数据。如果系统开启了多张物理或虚拟网卡，包括 WSL、Docker、VPN、虚拟交换机等，列表中可能包含这些额外网络条目。

* **Windows 首要支持**
  当前项目专门针对 Windows 系统的 Registry、IP Helper 与系统托盘行为进行定制设计。其他操作系统暂不作为首要支持和验证平台。

* **检测结果不等于合规结论**
  RouteLight 只显示网络状态和服务连通性结果，不判断用户网络接入方式、访问行为或组织策略是否合规。

---

## 📄 License

MIT License. See the repository-level `LICENSE` file.
