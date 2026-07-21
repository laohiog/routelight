# RouteLight v0.3.0 Release Notes

发布日期：2026-07-21

## 主要更新

- 将总体状态单灯替换为一个更醒目的复合托盘图标，竖排显示三条横向圆角状态带。
- 三条状态带从上到下固定对应 `ChatGPT`、`Claude` 和 `Google AI`，可直接判断具体异常服务。
- 托盘图标按服务名称匹配检测结果，不依赖数组顺序；缺少或尚未完成的结果显示灰色，额外服务不进入图标。
- 绿色表示当前探测可达或可用，黄色表示响应异常或需要人工确认，红色表示不可达或地区不支持，灰色表示未知或未完成。
- 新增动态托盘提示，固定列出三项服务短状态和总体路由状态；IPv4、IPv6、CN 出口等非 AI 风险仍保留在总体状态、通知和主面板中。
- 图标和提示在初次创建、手动刷新、托盘菜单刷新与 60 秒后台刷新时同步更新。

## 托盘状态预览

![正常、单项异常与缩放预览](https://github.com/laohiog/routelight/releases/download/v0.3.0/routelight-tray-capsule-contact-sheet.png)

固定顺序为 ChatGPT、Claude、Google AI。绿色只代表最近一次探测在当前网络路径下可达或可用，不代表账号、对话、API 或第三方服务的全部能力均正常。

## 兼容性与安全边界

- 未修改三个网络探针、Tauri commands、前端事件载荷、刷新频率和通知逻辑。
- 未新增系统权限、网络端点、持久化配置或用户设置。
- RouteLight 仍然只做只读诊断，不修改系统代理或路由表，不抓包，也不提供代理或 VPN 功能。

## 验证

- Rust 单元测试覆盖状态映射、正常与异常 HTTP、服务缺失、服务乱序、固定位置、透明背景、像素边界和 tooltip 长度。
- `normal`、`warning`、`error`、`unknown` mock 状态完成原生托盘人工验收。
- 已通过 Rust 格式检查、严格 Clippy、Rust 测试、前端语法检查和完整 Windows Tauri / NSIS 构建。

## 发布产物

- Windows NSIS 安装包：`RouteLight_0.3.0_x64-setup.exe`
- SHA256：`FBE7C8C5AE335BE48648E82CA4118BA1FBB93B3F0A5C493314A4507EC418C88A`

## 已知限制

- 当前推荐和验证的平台是 Windows，默认只发布 x64 NSIS 安装包。
- Windows 任务栏会按系统缩放设置重采样 32×32 图标，不同主题、显示器和 DPI 下可能存在轻微边缘差异。
- 后台检测默认每 60 秒刷新一次；托盘颜色表示最近一次检测结果，不是毫秒级实时状态。
