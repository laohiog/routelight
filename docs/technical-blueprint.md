RouteLight 项目 PRD + 技术实施蓝图

> **AI 代理路由状态灯：快速判断当前网络出口能否连接 ChatGPT、Claude，并使用 Google AI。**

---

# 一、项目定位

## 1. 项目名称建议

可以先用工作名：

```text
RouteLight
AI Route Monitor
VPN Status Lite
Proxy Route Status
```

我更推荐 **RouteLight**，因为它表达的是“路由状态灯”，不局限于 VPN。

---

## 2. 核心目标

这个工具解决 4 个问题：

```text
1. 当前公网出口 IP 是哪里？
2. 当前 IPv6 有没有绕过代理的风险？
3. ChatGPT / Claude 是否可达，Google AI 是否对当前出口开放？
4. 出现异常时，能不能一键复制诊断信息给 AI 分析？
```

它不应该承诺：

```text
账号绝对安全
IP 绝对纯净
VPN 绝对没泄露
OpenAI / Claude 永远不会风控
```

正确定位是：

```text
代理状态提示器
AI 服务连通性检测器
出口 IP 变化提醒器
网络诊断信息收集器
```

---

# 二、最终推荐技术栈

## 1. 推荐方案

```text
Tauri 2 + Rust 后端 + Vanilla HTML/CSS/JS 前端
Windows 优先
托盘常驻
点击托盘弹出小面板
```

Tauri 2 支持系统托盘，官方文档说明可以创建和定制系统托盘，并通过 `tray-icon` feature 启用相关能力。([Tauri][1]) Tauri 的桌面架构是 Rust 后端 + WebView 渲染 HTML 前端，前后端通过消息机制通信，适合这种“小 UI + 系统能力 + 网络检测”的工具。([Tauri][2])

Tauri 2 还有 capabilities 权限机制，可以控制哪些窗口或 WebView 能访问哪些能力，这一点适合做“只读检测工具”，避免前端随意调用系统能力。([Tauri][3])

---

## 2. 为什么不优先 Electron

Electron 也能做，而且跨平台成熟；Electron 官方说明它通过嵌入 Chromium 和 Node.js 来构建跨平台桌面应用。([Electron][4])

但这个项目的特点是：

```text
常驻后台
UI 很小
检测逻辑比界面更重要
不需要复杂前端生态
希望轻量、低资源占用
```

所以更适合 Tauri。

Electron 可以作为备选：

```text
如果你只想最快做原型：Electron
如果你想长期自用、轻量常驻：Tauri
```

---

# 三、MVP 功能边界

第一版只做这些：

```text
1. 托盘常驻
2. 点击托盘弹出状态面板
3. 手动刷新
4. 60 秒自动刷新
5. 检测 IPv4 出口 IP
6. 检测 IPv6 是否存在
7. 显示国家 / 城市 / ASN / 运营商
8. 检测 chatgpt.com 是否可达
9. 检测 claude.ai 是否可达
10. 匿名检测 google.com/ai 是否向当前出口开放 AI Mode
11. 读取 Windows 系统代理状态
12. 检测疑似 TUN / Wintun / Clash / Mihomo / v2rayN 网络接口
13. 出口 IP 变化时提示
14. 一键复制诊断信息
```

第一版不要做：

```text
纯净度评分
自动切换节点
自动修改系统代理
抓包
WebRTC 泄露检测
Canvas / WebGL 指纹检测
账号风控评分
复杂 DNS 泄露分析
跨平台完整适配
```

---

# 四、整体架构

## 1. 模块结构

```text
routelight/
├─ src/                          # 前端 UI
│  ├─ index.html
│  ├─ main.js
│  └─ styles.css
│
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs                 # Tauri 主入口、托盘、命令注册
│  │  ├─ app_state.rs            # 全局状态
│  │  ├─ config.rs               # 配置读取/保存
│  │  ├─ probe/
│  │  │  ├─ mod.rs
│  │  │  ├─ ip_probe.rs          # IPv4 / IPv6 出口检测
│  │  │  ├─ ai_probe.rs          # ChatGPT / Claude 连通性与 Google AI 可用性
│  │  │  ├─ local_probe.rs       # 系统代理、网卡、DNS
│  │  │  └─ score.rs             # 状态归类，不做纯净度评分
│  │  ├─ diagnostics.rs          # 诊断信息生成
│  │  ├─ notification.rs         # 系统通知
│  │  └─ storage.rs              # 本地历史记录
│  │
│  ├─ capabilities/
│  │  └─ default.json
│  │
│  ├─ icons/
│  │  ├─ green.ico
│  │  ├─ yellow.ico
│  │  ├─ red.ico
│  │  └─ gray.ico
│  │
│  └─ tauri.conf.json
│
├─ package.json
└─ README.md
```

---

## 2. 数据流

```text
定时器 / 手动刷新
        ↓
Rust 后端 probe_all()
        ↓
并发执行：
- 本机代理检测
- IPv4 出口检测
- IPv6 出口检测
- AI 服务连通性与 Google AI 可用性检测
        ↓
生成 RouteStatus
        ↓
状态判断：
绿色 / 黄色 / 红色 / 灰色
        ↓
更新：
- 前端面板
- 托盘图标
- Tooltip
- 本地历史
- 必要时系统通知
```

---

# 五、检测设计

## 1. 出口 IP 检测

### 检测内容

```text
IPv4 出口 IP
IPv6 出口 IP
国家
城市
ASN
运营商
IP 类型，若 API 支持
```

### 推荐数据源

第一版可以使用：

```text
主 IP 检测：ipify
备用 IP 检测：Cloudflare /cdn-cgi/trace
IP 信息：ipinfo / ipapi / ipwho.is 任选一个
```

ipify 官方支持返回公网 IP，格式包括纯文本、JSON 和 JSONP。([ipify][5]) Cloudflare 官方说明 `/cdn-cgi/trace` 是 Cloudflare 管理的 endpoint，可用于识别服务请求的数据中心并辅助排障。([Cloudflare Docs][6])

### MVP 推荐逻辑

```text
IPv4：
请求 IPv4-only 或普通 IP API

IPv6：
请求 IPv6-capable API
如果返回 IPv6，说明当前环境存在 IPv6 出口
如果 IPv4 是代理出口，IPv6 是本地/中国大陆出口，则提示风险
```

### 注意

不要把 IP 查询失败直接判定为 VPN 异常。

正确逻辑是：

```text
IP 查询失败 = 检测失败
AI 服务不可达 = 目标服务不可达
出口在中国大陆 = 高风险
IPv6 与 IPv4 地区不一致 = 注意
```

---

## 2. AI 服务连通性与 Google AI 可用性检测

检测目标：

```text
chatgpt.com
claude.ai
google.com/ai
```

检测方式：

```text
DNS 解析
TCP 连接
TLS 握手
HTTP 请求
响应状态码
耗时
错误类型
```

不要只看 HTTP 状态码。ChatGPT 与 Claude 检查网络连通性；Google AI 还需要检查最终跳转地址和明确的地区限制页面。

### 判断规则

```text
HTTP 200 / 301 / 302：
可达

HTTP 403：
网络可达，但服务拒绝，可能是地区、风控、权限或访问方式问题

Google AI 跳转到 `/search?udm=50`：
当前匿名网络路径可进入 AI Mode

Google 明确显示地区/语言不支持：
当前出口地区不支持

Google 验证码、Consent 或账号/设备限制：
需人工确认，不能直接判定为地区不支持

Timeout：
不可达

DNS failed：
DNS 异常

TLS failed：
TLS / SNI / 中间人 / 代理异常

Connection reset：
可能被阻断、代理异常或目标拒绝连接
```

### 推荐显示

```text
ChatGPT：可用 · 183ms
Claude：可用 · 205ms
Google AI：可用 · 200 · 192ms
```

不要简单显示：

```text
Google AI：失败
```

因为 Google 即使返回 HTTP 200，也可能展示地区不支持或需验证页面。

---

## 3. 本机代理状态检测

Windows 第一版检测：

```text
系统代理是否开启
HTTP Proxy
HTTPS Proxy
SOCKS Proxy，若可读到
PAC URL，若存在
疑似代理软件进程，选做
疑似 TUN / Wintun 网卡
DNS 服务器
默认网关
```

疑似网卡关键词：

```text
tun
tap
wintun
wireguard
clash
mihomo
meta
v2ray
sing-box
tailscale
zerotier
openvpn
```

注意：这些只能作为辅助信息。

不要写：

```text
检测到 Wintun，所以 VPN 正常
```

应该写：

```text
本机状态：检测到疑似 TUN 网卡
```

---

## 4. IPv6 风险检测

IPv6 是这个工具必须关注的点。

检测逻辑：

```text
1. 查询 IPv4 出口
2. 查询 IPv6 出口
3. 如果没有 IPv6：显示“未检测到 IPv6”
4. 如果有 IPv6：
   - 显示 IPv6 地址和地区
   - 对比 IPv4 与 IPv6 国家/ASN
   - 如果 IPv4 非中国大陆，IPv6 是中国大陆或本地运营商，显示黄色/红色风险
```

显示方式：

```text
IPv6：未检测到
IPv6：存在 · US · ASxxxx
IPv6：风险 · CN · 中国电信 / 中国移动 / 中国联通
```

---

# 六、状态判断规则

## 1. 总体状态

```text
绿色：正常
黄色：注意
红色：异常
灰色：检测失败 / 网络不可用
```

---

## 2. 绿色规则

满足：

```text
IPv4 出口非中国大陆
ChatGPT 可达
Claude 可达
Google AI 可用
未检测到明显 IPv6 直连风险
```

显示：

```text
状态：正常
```

---

## 3. 黄色规则

任一满足：

```text
IPv4 出口正常，但 Claude 不可达
IPv4 出口正常，但 ChatGPT 不可达
Google AI 需人工确认或无法判定
检测到 IPv6，但 IPv6 与 IPv4 出口地区/ASN 不一致
系统代理关闭，但出口 IP 仍然是代理出口
检测接口部分失败
```

显示：

```text
状态：注意
```

---

## 4. 红色规则

任一满足：

```text
IPv4 出口为中国大陆
ChatGPT / Claude / Google AI 全部不可达
Google 明确返回 AI Mode 地区不支持
IPv6 出口为中国大陆且 IPv6 可用
DNS 全部失败
网络不可用
```

显示：

```text
状态：异常
```

---

## 5. 灰色规则

```text
外部检测接口全部失败
网络断开
工具无法完成检测
```

显示：

```text
状态：检测失败
```

---

# 七、UI 设计

## 1. 托盘图标

```text
绿色：正常
黄色：注意
红色：异常
灰色：检测失败
```

Tooltip 示例：

```text
RouteLight
状态：正常
出口：US · Los Angeles
IP：xxx.xxx.xxx.xxx
ChatGPT：OK
Claude：OK
上次检测：16:42:18
```

---

## 2. 弹出面板

尺寸建议：

```text
宽度：320–380 px
高度：420–520 px
风格：极简、深浅主题均可
```

面板示例：

```text
RouteLight

状态       正常
出口       US · Los Angeles
IP         xxx.xxx.xxx.xxx
ASN        ASxxxx · Example ISP
IPv6       未检测到

ChatGPT       可用 · 183ms
Claude        可用 · 205ms
Google AI     可用 · 200 · 192ms

系统代理   127.0.0.1:7890
TUN        检测到 Wintun / Mihomo
DNS        1.1.1.1 / 8.8.8.8

上次检测   2026-06-28 16:42:18

[刷新] [复制诊断信息] [设置]
```

---

## 3. 设置页

第一版设置项不要太多：

```text
刷新间隔：
15 秒 / 30 秒 / 60 秒 / 300 秒

启动时最小化到托盘：
开 / 关

出口变化通知：
开 / 关

AI 服务不可达通知：
开 / 关

IPv6 风险通知：
开 / 关

IP 信息服务：
自动 / ipinfo / ipapi / ipwho.is

主题：
跟随系统 / 浅色 / 深色
```

---

# 八、配置文件设计

示例：

```json
{
  "refresh_interval_seconds": 60,
  "notify_on_ip_change": true,
  "notify_on_ai_unreachable": true,
  "notify_on_ipv6_risk": true,
  "theme": "system",
  "ip_providers": {
    "ipv4": [
      "https://api.ipify.org?format=json"
    ],
    "ipv6": [
      "https://api64.ipify.org?format=json"
    ],
    "trace": [
      "https://www.cloudflare.com/cdn-cgi/trace"
    ]
  },
  "ai_targets": [
    {
      "name": "ChatGPT",
      "url": "https://chatgpt.com",
      "method": "GET",
      "expected": ["200", "301", "302", "403"]
    },
    {
      "name": "Claude",
      "url": "https://claude.ai",
      "method": "GET",
      "expected": ["200", "301", "302", "403"]
    },
    {
      "name": "Google AI",
      "url": "https://www.google.com/ai?hl=en",
      "method": "GET",
      "expected": ["ai_mode_redirect", "region_restricted", "manual_check"]
    }
  ],
  "local_adapter_keywords": [
    "tun",
    "tap",
    "wintun",
    "wireguard",
    "clash",
    "mihomo",
    "v2ray",
    "sing-box",
    "openvpn",
    "tailscale"
  ]
}
```

注意：第一版可以先把检测 URL 固定写死，不急着开放用户自定义，避免安全风险。

---

# 九、核心数据结构

## 1. 总状态

```ts
type OverallStatus = "normal" | "warning" | "error" | "unknown";

interface RouteStatus {
  overall: OverallStatus;
  checkedAt: string;
  ipv4?: IpResult;
  ipv6?: IpResult;
  ai: AiProbeResult[];
  local: LocalNetworkStatus;
  warnings: string[];
  errors: string[];
}
```

---

## 2. IP 结果

```ts
interface IpResult {
  address: string;
  version: "ipv4" | "ipv6";
  country?: string;
  region?: string;
  city?: string;
  asn?: string;
  org?: string;
  provider: string;
  latencyMs?: number;
  error?: string;
}
```

---

## 3. AI 服务检测结果

```ts
interface AiProbeResult {
  name: string;
  url: string;
  reachable: boolean;
  probeStatus: "reachable" | "available" | "region_restricted" | "manual_check" | "unreachable" | "unknown";
  statusCode?: number;
  latencyMs?: number;
  phase: "dns" | "tcp" | "tls" | "http" | "done";
  errorType?: "dns_failed" | "timeout" | "tls_failed" | "connection_reset" | "http_error" | "unknown";
  message?: string;
}
```

---

## 4. 本机网络状态

```ts
interface LocalNetworkStatus {
  systemProxyEnabled: boolean;
  proxyServer?: string;
  pacUrl?: string;
  detectedAdapters: string[];
  dnsServers: string[];
  defaultGateway?: string;
}
```

---

# 十、诊断信息设计

用户点击“复制诊断信息”后，生成纯文本：

```text
RouteLight 诊断信息
时间：2026-06-28 16:42:18
总体状态：注意

[出口 IP]
IPv4：104.xxx.xxx.xxx
IPv4 地区：US / Los Angeles
IPv4 ASN：ASxxxx Example ISP
IPv6：未检测到

[AI 服务]
ChatGPT：可达，HTTP 200，183ms
Claude：不可达，timeout
Google AI：可用，HTTP 200，192ms

[本机代理]
系统代理：开启
代理地址：127.0.0.1:7890
PAC：无
疑似 TUN 网卡：Wintun / Mihomo
DNS：1.1.1.1, 8.8.8.8

[风险提示]
- Claude 页面不可达
- 未检测到 IPv6 风险

[最近出口变化]
16:20:12  104.xxx.xxx.xxx  US
15:58:02  172.xxx.xxx.xxx  SG
```

这个功能对你后续排查 v2rayN、Mihomo、Clash、Codex、Claude Code 连接问题非常实用。

---

# 十一、开发阶段拆分

## 阶段 0：项目初始化

目标：跑起来。

任务：

```text
1. 创建 Tauri 2 项目
2. 使用 Vanilla HTML/CSS/JS
3. 启用 tray-icon
4. 创建主窗口
5. 创建托盘菜单
6. 点击托盘显示/隐藏窗口
7. 准备 4 个状态图标
```

验收标准：

```text
可以运行桌面应用
任务栏托盘出现图标
点击托盘可以打开小面板
托盘菜单包含：
- 打开
- 刷新
- 复制诊断信息
- 退出
```

---

## 阶段 1：检测核心

目标：先不做漂亮 UI，只把检测结果跑通。

任务：

```text
1. 实现 IPv4 出口检测
2. 实现 IPv6 出口检测
3. 实现 AI 服务检测
4. 实现超时控制
5. 实现错误分类
6. 实现并发检测
7. 生成统一 RouteStatus
```

建议超时：

```text
单个 IP API：5 秒
单个 AI 目标：8 秒
全部检测总超时：15 秒
```

验收标准：

```text
点击刷新后，可以看到：
- IPv4
- IPv6 状态
- ChatGPT 检测结果
- Claude 检测结果
- Google AI 可用性检测结果
```

---

## 阶段 2：本机网络检测

目标：增加本地辅助判断。

任务：

```text
1. 读取 Windows 系统代理
2. 读取网卡列表
3. 匹配疑似 TUN / Wintun / 代理网卡
4. 读取 DNS 服务器，若可行
5. 读取默认网关，若可行
```

验收标准：

```text
面板显示：
系统代理：开启 / 关闭
代理地址：xxx
疑似代理网卡：xxx
DNS：xxx
```

---

## 阶段 3：状态判断和托盘颜色

目标：从“数据显示”变成“状态灯”。

任务：

```text
1. 实现 normal / warning / error / unknown
2. 根据状态切换托盘图标
3. 更新 Tooltip
4. 生成 warnings 和 errors
5. IP 变化时生成通知
6. AI 服务不可达时生成通知
```

验收标准：

```text
代理正常时显示绿色
出口回到中国大陆时显示红色
Claude 不可达但 ChatGPT 可达时显示黄色
检测失败时显示灰色
```

---

## 阶段 4：UI 打磨

目标：变成可长期使用的小工具。

任务：

```text
1. 精简面板布局
2. 增加刷新按钮
3. 增加复制诊断信息按钮
4. 增加上次检测时间
5. 增加最近出口变化记录
6. 增加设置页
7. 增加浅色/深色主题
```

验收标准：

```text
不需要打开浏览器
从托盘即可判断当前状态
异常时能一键复制诊断信息
```

---

## 阶段 5：高级分流检测，后续再做

目标：检测 GPT / Claude 是否走不同出口。

这一步需要自建检测端点：

```text
default-check.yourdomain.com
gpt-check.yourdomain.com
claude-check.yourdomain.com
```

然后在代理软件里配置：

```text
gpt-check.yourdomain.com     走 OpenAI 策略组
claude-check.yourdomain.com  走 Claude 策略组
default-check.yourdomain.com 走默认代理策略组
```

工具分别访问这几个域名，得到不同策略组的出口 IP。

但第一版不要做这个，避免复杂化。

---

# 十二、自建检测端点方案，后续版本

如果后面要做高级版，可以用 Cloudflare Worker。

返回格式：

```json
{
  "ip": "104.xxx.xxx.xxx",
  "country": "US",
  "colo": "LAX",
  "userAgent": "RouteLight/0.2.0",
  "timestamp": 1780000000
}
```

注意：

```text
1. 不记录用户历史
2. 不记录完整 User-Agent，或只做本地显示
3. 不保存访问日志，若平台允许
4. 每个检测域名只用于分流验证
```

高级版 UI：

```text
默认出口：US · 104.xxx.xxx.xxx
GPT 出口：US · 172.xxx.xxx.xxx
Claude 出口：SG · 138.xxx.xxx.xxx
国内直连：CN · 本地运营商
```

但要明确提示：

```text
分流检测需要用户在代理规则中配置检测域名。
未配置时，检测结果不代表真实 GPT / Claude 出口。
```

---

# 十三、安全与隐私原则

第一版必须遵守：

```text
只读，不修改系统代理
只检测，不抓包
不读取浏览器 Cookie
不读取代理软件配置
不上传本机网络详情
不上传诊断报告
不保存敏感信息
不默认开机自启
不默认自动更新
不访问非白名单 URL
```

Tauri 的能力权限要收紧。只允许前端调用必要命令：

```text
get_status
refresh_status
copy_diagnostics
open_settings
save_settings
```

不要开放：

```text
任意 shell 命令
任意文件读取
任意 URL 请求
修改系统代理
修改 hosts
修改 Clash / v2rayN 配置
```

---

# 十四、错误处理规则

## 1. 外部接口失败

显示：

```text
IP 检测失败
```

不要显示：

```text
VPN 失败
```

---

## 2. AI 服务失败

显示具体原因：

```text
ChatGPT：不可达 · timeout
Claude：不可达 · DNS failed
Google AI：需人工确认 · Google verification required
```

---

## 3. IPv6 检测失败

显示：

```text
IPv6：检测失败
```

不要显示：

```text
IPv6：安全
```

---

## 4. 网络断开

显示：

```text
状态：检测失败
原因：网络不可用
```

---

# 十五、验收测试清单

## 1. 正常代理场景

条件：

```text
v2rayN / Clash / Mihomo 正常开启
出口为美国 / 日本 / 新加坡
ChatGPT 和 Claude 可打开
```

期望：

```text
状态：绿色
ChatGPT：可达
Claude：可达
IPv6：无风险或未检测到
```

---

## 2. 关闭代理

条件：

```text
关闭代理软件
或者关闭系统代理
```

期望：

```text
出口显示中国大陆或本地运营商
状态：红色
ChatGPT / Claude 可能不可达
```

---

## 3. IPv6 风险场景

条件：

```text
IPv4 走代理
IPv6 直连中国大陆运营商
```

期望：

```text
状态：黄色或红色
提示：检测到 IPv6 直连风险
```

---

## 4. Google AI 地区不支持

条件：

```text
google.com/ai 返回明确的地区或语言不支持页面，即使 HTTP 状态为 200
```

期望：

```text
Google AI：地区不支持
状态：红色
不要因为 HTTP 200 而标记为可用
```

---

## 5. Claude 单独不可达

条件：

```text
ChatGPT 可用
Claude 不可用
```

期望：

```text
状态：黄色
提示：Claude 不可达
```

---

## 6. 检测接口失败

条件：

```text
ipify 不可用
备用接口可用
```

期望：

```text
自动使用备用接口
状态不应直接红色
```

---

# 十六、开发优先级

## P0：必须做

```text
托盘
手动刷新
IPv4 出口
IPv6 检测
ChatGPT 检测
Claude 检测
Google AI 可用性检测
状态灯
复制诊断信息
```

## P1：应该做

```text
系统代理读取
疑似 TUN 网卡检测
IP 变化通知
AI 不可达通知
刷新间隔设置
最近出口历史
```

## P2：后续做

```text
自建分流检测端点
多策略组出口显示
配置导入导出
macOS 支持
Linux 支持
自动更新
代码签名
```

## 暂不做

```text
纯净度评分
节点推荐
自动切换代理
代理软件控制面板
账号风控判断
浏览器指纹检测
```

最关键的是：**第一版一定要克制**。

不要一开始做成：

```text
VPN 管理器
节点管理器
代理切换器
IP 风控评分系统
浏览器指纹检测站
```

而是只做：

```text
当前出口是谁
AI 服务通不通
IPv6 有没有风险
异常时能不能快速诊断
```

这样它会非常实用，而且开发风险最低。

[1]: https://v2.tauri.app/learn/system-tray/?utm_source=chatgpt.com "System Tray"
[2]: https://v2.tauri.app/concept/architecture/?utm_source=chatgpt.com "Tauri Architecture"
[3]: https://v2.tauri.app/security/capabilities/?utm_source=chatgpt.com "Capabilities"
[4]: https://electronjs.org/docs/latest?utm_source=chatgpt.com "Introduction"
[5]: https://www.ipify.org/?utm_source=chatgpt.com "ipify - A Simple Public IP Address API"
[6]: https://developers.cloudflare.com/fundamentals/reference/cdn-cgi-endpoint/?utm_source=chatgpt.com "cdn-cgi/ endpoint - Cloudflare Fundamentals"
