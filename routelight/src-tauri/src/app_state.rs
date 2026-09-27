use serde::{Deserialize, Serialize};
use std::env;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OverallStatus {
    Normal,
    Warning,
    Error,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiProbeStatus {
    Reachable,
    Available,
    RegionRestricted,
    ManualCheck,
    Unreachable,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiServiceResult {
    pub name: String,
    pub url: String,
    pub reachable: bool,
    pub probe_status: AiProbeStatus,
    pub status_code: Option<u16>,
    pub latency_ms: Option<u64>,
    pub error_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteStatus {
    pub overall: OverallStatus,
    pub checked_at: String,
    pub ipv4: String,
    pub ipv6: String,
    pub country: String,
    pub city: String,
    pub asn: String,
    pub isp: String,
    pub ai_services: Vec<AiServiceResult>,
    pub local_proxy: String,
    pub tun_adapters: Vec<String>,
    pub dns_servers: Vec<String>,
    pub gateways: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IpChangeEntry {
    pub timestamp: String,
    pub old_ip: String,
    pub new_ip: String,
    pub country: String,
    pub asn: String,
}

static IP_HISTORY: Mutex<Vec<IpChangeEntry>> = Mutex::new(Vec::new());
static LAST_IP: Mutex<Option<(String, String, String)>> = Mutex::new(None); // (ip, country, asn)
static IS_REFRESHING: AtomicBool = AtomicBool::new(false);
static CURRENT_STATUS: Mutex<Option<RouteStatus>> = Mutex::new(None);

struct RefreshGuard;

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        IS_REFRESHING.store(false, Ordering::SeqCst);
    }
}

pub fn record_ip_if_changed(new_ip: &str, new_country: &str, new_asn: &str, timestamp: &str) {
    if new_ip == "Detection failed" || new_ip.is_empty() {
        return;
    }

    let mut last_ip_lock = LAST_IP.lock().unwrap();
    let mut history_lock = IP_HISTORY.lock().unwrap();

    if let Some((old_ip, _old_country, _old_asn)) = &*last_ip_lock {
        if old_ip != new_ip {
            let entry = IpChangeEntry {
                timestamp: timestamp.to_string(),
                old_ip: old_ip.clone(),
                new_ip: new_ip.to_string(),
                country: new_country.to_string(),
                asn: new_asn.to_string(),
            };
            history_lock.push(entry);
            if history_lock.len() > 20 {
                history_lock.remove(0);
            }
            *last_ip_lock = Some((
                new_ip.to_string(),
                new_country.to_string(),
                new_asn.to_string(),
            ));
        }
    } else {
        // First initialization
        *last_ip_lock = Some((
            new_ip.to_string(),
            new_country.to_string(),
            new_asn.to_string(),
        ));
    }
}

pub fn get_ai_status_label(service: &AiServiceResult) -> String {
    match service.probe_status {
        AiProbeStatus::Available => return "可用".to_string(),
        AiProbeStatus::RegionRestricted => return "地区不支持".to_string(),
        AiProbeStatus::ManualCheck => return "需人工确认".to_string(),
        AiProbeStatus::Unreachable => return "不可达".to_string(),
        AiProbeStatus::Unknown => return "无法判定".to_string(),
        AiProbeStatus::Reachable => {}
    }

    if let Some(code) = service.status_code {
        match code {
            200 | 301 | 302 => "可达".to_string(),
            403 => "可达但受限".to_string(),
            400 | 401 => "HTTP 可达但响应受限".to_string(),
            404 => "HTTP 可达但端点无效".to_string(),
            405 => "HTTP 可达但方法不允许".to_string(),
            _ => "HTTP 可达但响应异常".to_string(),
        }
    } else {
        "可达".to_string()
    }
}

pub fn get_mock_status() -> RouteStatus {
    let mock_env = env::var("ROUTELIGHT_MOCK_STATUS").unwrap_or_else(|_| "unknown".to_string());
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let status = match mock_env.to_lowercase().as_str() {
        "normal" => RouteStatus {
            overall: OverallStatus::Normal,
            checked_at: now.clone(),
            ipv4: "104.16.0.1".to_string(),
            ipv6: "Not detected".to_string(),
            country: "US".to_string(),
            city: "Los Angeles".to_string(),
            asn: "AS13335".to_string(),
            isp: "Cloudflare, Inc.".to_string(),
            ai_services: vec![
                AiServiceResult {
                    name: "ChatGPT".to_string(),
                    url: "https://chatgpt.com".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::Reachable,
                    status_code: Some(200),
                    latency_ms: Some(183),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::Reachable,
                    status_code: Some(200),
                    latency_ms: Some(205),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "Google AI".to_string(),
                    url: "https://www.google.com/ai?hl=en".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::Available,
                    status_code: Some(200),
                    latency_ms: Some(192),
                    error_type: Some("MOCK_DATA".to_string()),
                },
            ],
            local_proxy: "127.0.0.1:7890".to_string(),
            tun_adapters: vec!["Wintun".to_string(), "Mihomo".to_string()],
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            gateways: vec!["192.168.1.1".to_string()],
            warnings: vec![],
            errors: vec![],
        },
        "warning" => RouteStatus {
            overall: OverallStatus::Warning,
            checked_at: now.clone(),
            ipv4: "104.16.0.1".to_string(),
            ipv6: "240e::1234 (CN)".to_string(),
            country: "US".to_string(),
            city: "Los Angeles".to_string(),
            asn: "AS13335".to_string(),
            isp: "Cloudflare, Inc.".to_string(),
            ai_services: vec![
                AiServiceResult {
                    name: "ChatGPT".to_string(),
                    url: "https://chatgpt.com".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::Reachable,
                    status_code: Some(200),
                    latency_ms: Some(183),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: false,
                    probe_status: AiProbeStatus::Unreachable,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("timeout".to_string()),
                },
                AiServiceResult {
                    name: "Google AI".to_string(),
                    url: "https://www.google.com/ai?hl=en".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::Available,
                    status_code: Some(200),
                    latency_ms: Some(192),
                    error_type: Some("MOCK_DATA".to_string()),
                },
            ],
            local_proxy: "127.0.0.1:7890".to_string(),
            tun_adapters: vec!["Wintun".to_string()],
            dns_servers: vec!["1.1.1.1".to_string()],
            gateways: vec!["192.168.1.1".to_string()],
            warnings: vec![
                "IPv6 direct-connect risk".to_string(),
                "Claude is unreachable: timeout".to_string(),
            ],
            errors: vec![],
        },
        "error" => RouteStatus {
            overall: OverallStatus::Error,
            checked_at: now.clone(),
            ipv4: "116.228.1.1 (CN)".to_string(),
            ipv6: "240e::1234 (CN)".to_string(),
            country: "CN".to_string(),
            city: "Shanghai".to_string(),
            asn: "AS4812".to_string(),
            isp: "China Telecom".to_string(),
            ai_services: vec![
                AiServiceResult {
                    name: "ChatGPT".to_string(),
                    url: "https://chatgpt.com".to_string(),
                    reachable: false,
                    probe_status: AiProbeStatus::Unreachable,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("connection reset".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: false,
                    probe_status: AiProbeStatus::Unreachable,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("DNS failed".to_string()),
                },
                AiServiceResult {
                    name: "Google AI".to_string(),
                    url: "https://www.google.com/ai?hl=en".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::RegionRestricted,
                    status_code: Some(200),
                    latency_ms: Some(192),
                    error_type: Some("region unsupported".to_string()),
                },
            ],
            local_proxy: "Disabled".to_string(),
            tun_adapters: vec![],
            dns_servers: vec!["223.5.5.5".to_string()],
            gateways: vec!["192.168.1.1".to_string()],
            warnings: vec![],
            errors: vec![
                "Google AI is unavailable for this region".to_string(),
                "exit appears local/CN (IPv4)".to_string(),
            ],
        },
        _ => RouteStatus {
            overall: OverallStatus::Unknown,
            checked_at: now.clone(),
            ipv4: "Detection failed".to_string(),
            ipv6: "Detection failed".to_string(),
            country: "Unknown".to_string(),
            city: "Unknown".to_string(),
            asn: "Unknown".to_string(),
            isp: "Unknown".to_string(),
            ai_services: vec![
                AiServiceResult {
                    name: "ChatGPT".to_string(),
                    url: "https://chatgpt.com".to_string(),
                    reachable: false,
                    probe_status: AiProbeStatus::Unknown,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("unknown".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: false,
                    probe_status: AiProbeStatus::Unknown,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("unknown".to_string()),
                },
                AiServiceResult {
                    name: "Google AI".to_string(),
                    url: "https://www.google.com/ai?hl=en".to_string(),
                    reachable: false,
                    probe_status: AiProbeStatus::Unknown,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("unknown".to_string()),
                },
            ],
            local_proxy: "Unknown".to_string(),
            tun_adapters: vec![],
            dns_servers: vec![],
            gateways: vec![],
            warnings: vec!["Mock unknown state".to_string()],
            errors: vec![],
        },
    };

    record_ip_if_changed(&status.ipv4, &status.country, &status.asn, &now);
    status
}

pub async fn get_real_status() -> RouteStatus {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let ip_res_fut = crate::probe::ip_probe::probe_ips();
    let ai_res_fut = crate::probe::ai_probe::probe_ai_services();

    // Spawn them on tokio threadpool or join them
    let (ip_res, ai_services) = tokio::join!(ip_res_fut, ai_res_fut);

    // Call local network adapter and proxy detection
    let local_net = crate::probe::local_probe::probe_local_network();

    let mut warnings = ip_res.warnings.clone();
    let mut errors = Vec::new();

    // Check Geolocation
    let mut country = "Unknown".to_string();
    let mut city = "Unknown".to_string();
    let mut asn = "Unknown".to_string();
    let mut isp = "Unknown".to_string();

    if let Some(ref geo) = ip_res.ipv4_geo {
        if let Some(ref cc) = geo.country_code {
            country = cc.clone();
            if cc == "CN" {
                errors.push("exit appears local/CN (IPv4)".to_string());
            }
        }
        if let Some(ref c) = geo.city {
            city = c.clone();
        }
        if let Some(ref conn) = geo.connection {
            if let Some(ref a) = conn.asn {
                asn = format!("AS{}", a);
            }
            if let Some(ref i) = conn.isp {
                isp = i.clone();
            }
        }
    } else {
        warnings.push("IPv4 geolocation query failed".to_string());
    }

    // Check IPv6 status
    let ipv6_str = match &ip_res.ipv6 {
        crate::probe::ip_probe::Ipv6Result::Success(addr) => {
            if let Some(ref v6_geo) = ip_res.ipv6_geo {
                if let Some(ref cc) = v6_geo.country_code {
                    if cc == "CN" {
                        errors.push("IPv6 exit country matches CN".to_string());
                    }
                }
            }
            addr.clone()
        }
        crate::probe::ip_probe::Ipv6Result::NotDetected => "未检测到 IPv6".to_string(),
        crate::probe::ip_probe::Ipv6Result::QueryFailed(err) => {
            warnings.push(format!("IPv6 查询失败: {}", err));
            format!("IPv6 查询失败: {}", err)
        }
    };

    // Geolocation mismatch check
    if let (Some(ref v4), Some(ref v6)) = (&ip_res.ipv4_geo, &ip_res.ipv6_geo) {
        if v4.country_code != v6.country_code {
            warnings.push("IPv4 and IPv6 exit countries are inconsistent".to_string());
        }
    }

    // Evaluate AI services. ChatGPT and Claude are connectivity probes; Google AI also
    // verifies whether Google exposes AI Mode to this anonymous network path.
    let chatgpt = ai_services.iter().find(|service| service.name == "ChatGPT");
    let claude = ai_services.iter().find(|service| service.name == "Claude");
    let google_ai = ai_services
        .iter()
        .find(|service| service.name == "Google AI");

    let chatgpt_reachable = chatgpt.map(|service| service.reachable).unwrap_or(false);
    let claude_reachable = claude.map(|service| service.reachable).unwrap_or(false);
    let google_reachable = google_ai.map(|service| service.reachable).unwrap_or(false);
    let google_available = google_ai
        .map(|service| service.probe_status == AiProbeStatus::Available)
        .unwrap_or(false);
    let all_ai_unreachable = !chatgpt_reachable && !claude_reachable && !google_reachable;

    if all_ai_unreachable {
        errors.push("All AI services are unreachable".to_string());
    } else {
        for (name, service) in [("ChatGPT", chatgpt), ("Claude", claude)] {
            match service {
                Some(service) if !service.reachable => {
                    let detail = service.error_type.as_deref().unwrap_or("unknown");
                    warnings.push(format!("{name} is unreachable: {detail}"));
                }
                Some(service) => {
                    if let Some(code) = service.status_code {
                        if code != 200 && code != 301 && code != 302 {
                            warnings.push(format!(
                                "{name} returned HTTP {code}; network is reachable but response is abnormal"
                            ));
                        }
                    }
                }
                None => warnings.push(format!("{name} probe returned no result")),
            }
        }
    }

    match google_ai.map(|service| &service.probe_status) {
        Some(AiProbeStatus::Available) => {}
        Some(AiProbeStatus::RegionRestricted) => {
            errors.push("Google AI is unavailable for this region".to_string());
        }
        Some(AiProbeStatus::ManualCheck) => {
            let detail = google_ai
                .and_then(|service| service.error_type.as_deref())
                .unwrap_or("manual verification required");
            warnings.push(format!("Google AI requires manual verification: {detail}"));
        }
        Some(AiProbeStatus::Unreachable) if !all_ai_unreachable => {
            let detail = google_ai
                .and_then(|service| service.error_type.as_deref())
                .unwrap_or("unknown");
            warnings.push(format!("Google AI is unreachable: {detail}"));
        }
        Some(AiProbeStatus::Unknown) | Some(AiProbeStatus::Reachable) => {
            let detail = google_ai
                .and_then(|service| service.error_type.as_deref())
                .unwrap_or("availability could not be determined");
            warnings.push(format!("Google AI availability is unknown: {detail}"));
        }
        Some(AiProbeStatus::Unreachable) => {}
        None => warnings.push("Google AI probe returned no result".to_string()),
    }

    // Determine Overall Status
    let overall = if ip_res.ipv4 == "Detection failed" {
        OverallStatus::Unknown
    } else if !errors.is_empty() {
        OverallStatus::Error
    } else if !warnings.is_empty() {
        OverallStatus::Warning
    } else {
        // Normal state rules
        let ipv4_non_cn = ip_res
            .ipv4_geo
            .as_ref()
            .map(|g| {
                g.country_code
                    .as_ref()
                    .map(|cc| cc != "CN")
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let ipv6_non_cn = match &ip_res.ipv6 {
            crate::probe::ip_probe::Ipv6Result::Success(_) => ip_res
                .ipv6_geo
                .as_ref()
                .map(|g| {
                    g.country_code
                        .as_ref()
                        .map(|cc| cc != "CN")
                        .unwrap_or(false)
                })
                .unwrap_or(false),
            _ => true, // Not present means no direct CN risk
        };

        if ipv4_non_cn && ipv6_non_cn && chatgpt_reachable && claude_reachable && google_available {
            OverallStatus::Normal
        } else {
            OverallStatus::Warning
        }
    };

    record_ip_if_changed(&ip_res.ipv4, &country, &asn, &now);

    RouteStatus {
        overall,
        checked_at: now,
        ipv4: ip_res.ipv4,
        ipv6: ipv6_str,
        country,
        city,
        asn,
        isp,
        ai_services,
        local_proxy: local_net.proxy,
        tun_adapters: local_net.tun_adapters,
        dns_servers: local_net.dns_servers,
        gateways: local_net.gateways,
        warnings,
        errors,
    }
}

pub async fn get_status_data() -> RouteStatus {
    if env::var("ROUTELIGHT_MOCK_STATUS").is_ok() {
        let status = get_mock_status();
        *CURRENT_STATUS.lock().unwrap() = Some(status.clone());
        return status;
    }

    if IS_REFRESHING.swap(true, Ordering::SeqCst) {
        if let Some(cached) = &*CURRENT_STATUS.lock().unwrap() {
            println!(
                "[app_state] duplicate concurrent refresh request ignored, returning cached status"
            );
            return cached.clone();
        }
    }

    let _guard = RefreshGuard;
    let status = get_real_status().await;
    *CURRENT_STATUS.lock().unwrap() = Some(status.clone());
    status
}

pub fn get_cached_status_data() -> Option<RouteStatus> {
    CURRENT_STATUS.lock().unwrap().clone()
}

pub async fn copy_diagnostics_data() -> Result<String, String> {
    let cached = {
        let cache = CURRENT_STATUS.lock().unwrap();
        cache.clone()
    };

    let status = match cached {
        Some(s) => s,
        None => get_status_data().await,
    };

    let text = generate_diagnostics_text(&status);
    if let Ok(mut ctx) = arboard::Clipboard::new() {
        if ctx.set_text(text.clone()).is_ok() {
            println!("[menu] copy diagnostics");
            return Ok(text);
        }
    }
    Err("Failed to write to clipboard".to_string())
}

pub async fn copy_ai_diagnostic_context_data() -> Result<String, String> {
    let status =
        get_cached_status_data().ok_or_else(|| "RouteLight status is not ready yet".to_string())?;
    let text = generate_ai_diagnostic_context_text(&status);
    if let Ok(mut ctx) = arboard::Clipboard::new() {
        if ctx.set_text(text.clone()).is_ok() {
            println!("[menu] copy AI diagnostic context");
            return Ok(text);
        }
    }
    Err("Failed to write to clipboard".to_string())
}

pub fn generate_diagnostics_text(status: &RouteStatus) -> String {
    let mut text = String::new();
    text.push_str("RouteLight 诊断信息\n");
    text.push_str("=====================================\n");
    text.push_str(&format!("时间：{}\n", status.checked_at));
    text.push_str(&format!("总体状态：{:?}\n\n", status.overall));

    text.push_str("[出口 IP]\n");
    text.push_str(&format!("IPv4：{}\n", status.ipv4));
    text.push_str(&format!(
        "IPv4 地区：{} / {}\n",
        status.country, status.city
    ));
    text.push_str(&format!("IPv4 ASN/ISP：{} {}\n", status.asn, status.isp));
    text.push_str(&format!("IPv6：{}\n\n", status.ipv6));

    text.push_str("[AI 服务]\n");
    for ai in &status.ai_services {
        let mut parts = Vec::new();
        let label = get_ai_status_label(ai);
        parts.push(label);

        if let Some(code) = ai.status_code {
            parts.push(format!("HTTP {}", code));
        }
        if let Some(lat) = ai.latency_ms {
            parts.push(format!("{}ms", lat));
        }
        if !matches!(
            ai.probe_status,
            AiProbeStatus::Reachable | AiProbeStatus::Available
        ) {
            if let Some(ref err) = ai.error_type {
                parts.push(err.clone());
            }
        }

        text.push_str(&format!("- {}：{}\n", ai.name, parts.join("，")));
    }
    text.push('\n');

    text.push_str("[最近出口 IP 变化历史]\n");
    let history = IP_HISTORY.lock().unwrap().clone();
    if history.is_empty() {
        text.push_str("- 无 IP 变化记录\n\n");
    } else {
        for entry in history.iter().rev() {
            text.push_str(&format!(
                "- {}：{} -> {} (国家：{}，ASN：{})\n",
                entry.timestamp, entry.old_ip, entry.new_ip, entry.country, entry.asn
            ));
        }
        text.push('\n');
    }

    text.push_str("[本机网络 / 代理]\n");
    text.push_str(&format!("系统代理：{}\n", status.local_proxy));
    text.push_str(&format!(
        "疑似 TUN / VPN / 虚拟网卡：{}\n",
        status.tun_adapters.join(", ")
    ));
    text.push_str(&format!(
        "DNS 服务器（原始）：{}\n",
        status.dns_servers.join(", ")
    ));
    text.push_str(&format!(
        "默认网关（原始）：{}\n\n",
        status.gateways.join(", ")
    ));

    text.push_str("[风险与错误]\n");
    let mut has_warnings_or_errors = false;
    for w in &status.warnings {
        text.push_str(&format!("- 警告: {}\n", w));
        has_warnings_or_errors = true;
    }
    for e in &status.errors {
        text.push_str(&format!("- 错误: {}\n", e));
        has_warnings_or_errors = true;
    }
    if !has_warnings_or_errors {
        text.push_str("- 无明显风险\n");
    }
    text.push('\n');

    text.push_str("[工具说明 / 已知限制]\n");
    text.push_str("- 本工具所显示的“系统代理”及“疑似虚拟网卡”信息仅作为本地网络拓扑的参考依据。\n");
    text.push_str("- 网络判定基于第三方 IP 地理库与实时端点握手，可能因服务商缓存、CDN 调度或临时网络抖动产生偏差。\n");
    text.push_str("- Google AI 使用无账号、无 Cookie 的匿名页面探测；账号、语言、实验分组或验证码仍可能影响浏览器中的实际结果。\n");

    text
}

fn ai_context_ipv6_state(value: &str) -> &'static str {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.contains("not detected") || value.contains("未检测到") {
        "not detected"
    } else if normalized.is_empty()
        || normalized.contains("failed")
        || value.contains("失败")
        || normalized.contains("unknown")
    {
        "query failed"
    } else {
        "detected"
    }
}

fn ai_context_proxy_state(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "unknown" => "unknown",
        "disabled" => "disabled",
        _ => "configured",
    }
}

fn add_ip_literals_to_redactions(value: &str, redactions: &mut Vec<String>) {
    let mut candidate = String::new();
    for character in value.chars().chain(std::iter::once(' ')) {
        if character.is_ascii_hexdigit() || matches!(character, '.' | ':' | '%') {
            candidate.push(character);
        } else {
            let address = candidate.split('%').next().unwrap_or_default();
            if let Ok(parsed) = address.parse::<IpAddr>() {
                redactions.push(address.to_string());
                redactions.push(parsed.to_string());
            }
            candidate.clear();
        }
    }
}

fn ai_context_redactions(status: &RouteStatus, history: &[IpChangeEntry]) -> Vec<String> {
    let mut redactions = Vec::new();
    add_ip_literals_to_redactions(&status.ipv4, &mut redactions);
    add_ip_literals_to_redactions(&status.ipv6, &mut redactions);

    if ai_context_proxy_state(&status.local_proxy) == "configured" {
        redactions.push(status.local_proxy.clone());
        if let Some(pac_index) = status.local_proxy.to_ascii_lowercase().find("pac:") {
            redactions.push(status.local_proxy[pac_index + 4..].trim().to_string());
        }
    }

    for gateway in &status.gateways {
        redactions.push(gateway.clone());
        add_ip_literals_to_redactions(gateway, &mut redactions);
    }
    for entry in history {
        redactions.push(entry.old_ip.clone());
        redactions.push(entry.new_ip.clone());
        add_ip_literals_to_redactions(&entry.old_ip, &mut redactions);
        add_ip_literals_to_redactions(&entry.new_ip, &mut redactions);
    }

    redactions.retain(|value| !value.is_empty());
    redactions.sort();
    redactions.dedup();
    redactions.sort_by(|left, right| right.len().cmp(&left.len()));
    redactions
}

fn redact_ai_context_value(value: &str, redactions: &[String]) -> String {
    redactions.iter().fold(value.to_string(), |text, value| {
        text.replace(value, "[已省略]")
    })
}

fn redact_ai_context_dns_value(value: &str, redactions: &[String]) -> String {
    if let Ok(address) = value.trim().parse::<IpAddr>() {
        let is_sensitive = redactions.iter().any(|redaction| {
            redaction == value || redaction.parse::<IpAddr>().ok() == Some(address)
        });
        if is_sensitive {
            "[已省略]".to_string()
        } else {
            value.to_string()
        }
    } else {
        redact_ai_context_value(value, redactions)
    }
}

pub fn generate_ai_diagnostic_context_text(status: &RouteStatus) -> String {
    let history = IP_HISTORY.lock().unwrap().clone();
    format_ai_diagnostic_context_with_history(status, &history)
}

fn format_ai_diagnostic_context_with_history(
    status: &RouteStatus,
    history: &[IpChangeEntry],
) -> String {
    let redactions = ai_context_redactions(status, history);
    let safe = |value: &str| redact_ai_context_value(value, &redactions);
    let mut text = String::new();

    text.push_str("RouteLight AI 诊断上下文\n");
    text.push_str("=====================================\n\n");
    text.push_str("[用户问题]\n");
    text.push_str("请在发送给 AI 前补充：实际症状、开始时间、影响范围。\n\n");

    text.push_str("[观察事实]\n");
    text.push_str(&format!("检测时间：{}\n", safe(&status.checked_at)));
    text.push_str(&format!("RouteLight 总体状态：{:?}\n", status.overall));
    text.push_str(&format!(
        "IPv4 地区：{} / {}\n",
        safe(&status.country),
        safe(&status.city)
    ));
    text.push_str(&format!(
        "ASN / ISP：{} / {}\n",
        safe(&status.asn),
        safe(&status.isp)
    ));
    text.push_str(&format!(
        "IPv6 状态：{}\n",
        ai_context_ipv6_state(&status.ipv6)
    ));

    for service in &status.ai_services {
        let mut details = vec![safe(&get_ai_status_label(service))];
        if let Some(code) = service.status_code {
            details.push(format!("HTTP {code}"));
        }
        if let Some(latency) = service.latency_ms {
            details.push(format!("{latency}ms"));
        }
        if !matches!(
            service.probe_status,
            AiProbeStatus::Reachable | AiProbeStatus::Available
        ) {
            if let Some(error_type) = &service.error_type {
                if error_type != "MOCK_DATA" {
                    details.push(safe(error_type));
                }
            }
        }
        text.push_str(&format!(
            "{}：{}\n",
            safe(&service.name),
            details.join("，")
        ));
    }

    text.push_str(&format!(
        "系统代理状态：{}\n",
        ai_context_proxy_state(&status.local_proxy)
    ));
    let adapters: Vec<String> = status
        .tun_adapters
        .iter()
        .map(|value| safe(value))
        .collect();
    text.push_str(&format!(
        "疑似 TUN / VPN / 虚拟网卡：{}\n",
        if adapters.is_empty() {
            "无".to_string()
        } else {
            adapters.join(", ")
        }
    ));
    let dns_servers: Vec<String> = status
        .dns_servers
        .iter()
        .map(|value| redact_ai_context_dns_value(value, &redactions))
        .collect();
    text.push_str(&format!(
        "DNS 服务器：{}\n\n",
        if dns_servers.is_empty() {
            "无".to_string()
        } else {
            dns_servers.join(", ")
        }
    ));

    text.push_str("[RouteLight 警告与错误]\n");
    if status.warnings.is_empty() && status.errors.is_empty() {
        text.push_str("- 无\n\n");
    } else {
        for warning in &status.warnings {
            text.push_str(&format!("- 警告：{}\n", safe(warning)));
        }
        for error in &status.errors {
            text.push_str(&format!("- 错误：{}\n", safe(error)));
        }
        text.push('\n');
    }

    text.push_str("[最近出口变化（原始 IP 已脱敏）]\n");
    if history.is_empty() {
        text.push_str("- 无内存记录\n\n");
    } else {
        for entry in history.iter().rev() {
            text.push_str(&format!(
                "- {}：检测到 IPv4 出口变化（前后地址已省略）；新出口地区：{}，ASN：{}\n",
                safe(&entry.timestamp),
                safe(&entry.country),
                safe(&entry.asn)
            ));
        }
        text.push('\n');
    }

    text.push_str("[隐私与脱敏说明]\n");
    text.push_str("此上下文由 RouteLight 在本地基于现有内存状态生成；只有在你显式点击“AI 上下文”后才复制到剪贴板。RouteLight 不会自动上传或向任何 AI 服务发送这些内容。\n");
    text.push_str("已省略敏感或不必要的原始网络标识：当前公网 IPv4 / IPv6、历史原始公网 IP、原始代理服务器字符串、PAC URL、默认网关。内容不包含凭据、令牌、订阅链接、Cookie 或本地文件内容。\n\n");

    text.push_str("[给 AI 的分析要求]\n");
    text.push_str("请分析以上上下文，并遵循以下要求：\n");
    text.push_str("1. 将观察事实与推断分开。\n");
    text.push_str("2. 列出不超过 3 个可能原因。\n");
    text.push_str("3. 为每个原因引用支持它的 RouteLight 证据。\n");
    text.push_str("4. 证据不足时明确说明。\n");
    text.push_str("5. 提出最小且有用的下一步诊断测试。\n");
    text.push_str(
        "6. 不要假设你已获准修改代理设置、路由、防火墙、VPN 客户端、NAS、VPS 或其他系统。\n",
    );

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ai_context_status() -> RouteStatus {
        RouteStatus {
            overall: OverallStatus::Warning,
            checked_at: "2026-09-27 10:00:00".to_string(),
            ipv4: "104.16.0.1".to_string(),
            ipv6: "240e:0000:0000:0000:0000:0000:0000:1234".to_string(),
            country: "US".to_string(),
            city: "Los Angeles".to_string(),
            asn: "AS13335".to_string(),
            isp: "Cloudflare, Inc.".to_string(),
            ai_services: vec![
                AiServiceResult {
                    name: "ChatGPT".to_string(),
                    url: "https://chatgpt.com".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::Reachable,
                    status_code: Some(200),
                    latency_ms: Some(183),
                    error_type: None,
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: false,
                    probe_status: AiProbeStatus::Unreachable,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("timeout".to_string()),
                },
                AiServiceResult {
                    name: "Google AI".to_string(),
                    url: "https://www.google.com/ai?hl=en".to_string(),
                    reachable: true,
                    probe_status: AiProbeStatus::Available,
                    status_code: Some(200),
                    latency_ms: Some(192),
                    error_type: None,
                },
            ],
            local_proxy: "127.0.0.1:7890".to_string(),
            tun_adapters: vec!["Wintun".to_string()],
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            gateways: vec!["192.168.1.1".to_string()],
            warnings: vec!["Claude is unreachable: timeout".to_string()],
            errors: vec![],
        }
    }

    fn sample_ai_context_history() -> Vec<IpChangeEntry> {
        vec![IpChangeEntry {
            timestamp: "2026-09-27 09:30:00".to_string(),
            old_ip: "198.51.100.10".to_string(),
            new_ip: "104.16.0.1".to_string(),
            country: "US".to_string(),
            asn: "AS13335".to_string(),
        }]
    }

    #[test]
    fn ai_context_contains_required_sections_facts_and_instructions() {
        let status = sample_ai_context_status();
        let history = sample_ai_context_history();
        let context = format_ai_diagnostic_context_with_history(&status, &history);

        for required in [
            "RouteLight AI 诊断上下文",
            "[用户问题]",
            "[观察事实]",
            "[最近出口变化（原始 IP 已脱敏）]",
            "[隐私与脱敏说明]",
            "[给 AI 的分析要求]",
            "请在发送给 AI 前补充：实际症状、开始时间、影响范围。",
            "ChatGPT",
            "Claude",
            "Google AI",
            "1.1.1.1, 8.8.8.8",
            "不超过 3 个可能原因",
            "证据不足时明确说明",
            "不要假设你已获准修改代理设置",
        ] {
            assert!(
                context.contains(required),
                "missing required text: {required}"
            );
        }
        assert!(context.contains("183ms"));
        assert!(context.contains("HTTP 200"));
        assert!(context.contains("timeout"));
        assert!(context.contains("IPv6 状态：detected"));
        assert!(context.contains("系统代理状态：configured"));
        assert!(context.contains("Wintun"));
    }

    #[test]
    fn ai_context_redacts_current_historical_proxy_pac_and_gateway_values() {
        let mut status = sample_ai_context_status();
        let history = sample_ai_context_history();
        let proxy = status.local_proxy.clone();
        let gateway = status.gateways[0].clone();
        status.dns_servers.push(status.ipv4.clone());
        status.warnings.push(format!(
            "Raw diagnostic details: {} {} {} {}",
            status.ipv4, status.ipv6, proxy, gateway
        ));

        let context = format_ai_diagnostic_context_with_history(&status, &history);
        for sensitive in [
            status.ipv4.as_str(),
            status.ipv6.as_str(),
            "240e::1234",
            proxy.as_str(),
            gateway.as_str(),
            history[0].old_ip.as_str(),
            history[0].new_ip.as_str(),
        ] {
            assert!(!context.contains(sensitive), "context exposed {sensitive}");
        }

        let pac_url = "https://proxy.example/config.pac?token=secret";
        status.local_proxy = format!("PAC: {pac_url}");
        status.warnings = vec![format!("PAC detail: {pac_url}")];
        let context = format_ai_diagnostic_context_with_history(&status, &history);
        assert!(!context.contains(pac_url));
        assert!(!context.contains("token=secret"));
    }

    #[test]
    fn ai_context_preserves_dns_address_sharing_a_gateway_prefix() {
        let mut status = sample_ai_context_status();
        status.dns_servers = vec!["192.168.1.10".to_string()];
        let history = sample_ai_context_history();

        let context = format_ai_diagnostic_context_with_history(&status, &history);

        assert!(context.contains("DNS 服务器：192.168.1.10"));
    }

    #[test]
    fn ai_context_reports_coarse_ipv6_and_proxy_states() {
        assert_eq!(ai_context_ipv6_state("Not detected"), "not detected");
        assert_eq!(
            ai_context_ipv6_state("IPv6 查询失败: timeout"),
            "query failed"
        );
        assert_eq!(ai_context_ipv6_state("240e::1234"), "detected");
        assert_eq!(ai_context_proxy_state("Disabled"), "disabled");
        assert_eq!(ai_context_proxy_state("Unknown"), "unknown");
        assert_eq!(ai_context_proxy_state("127.0.0.1:7890"), "configured");
    }

    #[test]
    fn mock_states_use_chatgpt_claude_and_google_ai_only() {
        let previous = env::var_os("ROUTELIGHT_MOCK_STATUS");

        for mock_state in ["normal", "warning", "error", "unknown"] {
            env::set_var("ROUTELIGHT_MOCK_STATUS", mock_state);
            let status = get_mock_status();
            let names: Vec<&str> = status
                .ai_services
                .iter()
                .map(|service| service.name.as_str())
                .collect();

            assert_eq!(names, vec!["ChatGPT", "Claude", "Google AI"]);
            let urls: Vec<&str> = status
                .ai_services
                .iter()
                .map(|service| service.url.as_str())
                .collect();
            assert_eq!(
                urls,
                vec![
                    "https://chatgpt.com",
                    "https://claude.ai",
                    "https://www.google.com/ai?hl=en"
                ]
            );
        }

        match previous {
            Some(value) => env::set_var("ROUTELIGHT_MOCK_STATUS", value),
            None => env::remove_var("ROUTELIGHT_MOCK_STATUS"),
        }
    }

    #[test]
    fn serializes_google_ai_probe_status_for_the_frontend() {
        assert_eq!(
            serde_json::to_string(&AiProbeStatus::RegionRestricted).unwrap(),
            "\"region_restricted\""
        );
        assert_eq!(
            serde_json::to_string(&AiProbeStatus::ManualCheck).unwrap(),
            "\"manual_check\""
        );
    }
}
