use serde::{Deserialize, Serialize};
use std::env;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiServiceResult {
    pub name: String,
    pub url: String,
    pub reachable: bool,
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

pub fn get_ai_status_label(reachable: bool, status_code: Option<u16>) -> String {
    if !reachable {
        return "不可达".to_string();
    }
    if let Some(code) = status_code {
        match code {
            200 | 301 | 302 => "可达".to_string(),
            401 => "API 可达但未认证".to_string(),
            403 => "可达但受限".to_string(),
            400 => "API 可达但请求无效".to_string(),
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
                    status_code: Some(200),
                    latency_ms: Some(183),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "OpenAI API".to_string(),
                    url: "https://api.openai.com/v1/models".to_string(),
                    reachable: true,
                    status_code: Some(401),
                    latency_ms: Some(221),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: true,
                    status_code: Some(200),
                    latency_ms: Some(205),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "Anthropic API".to_string(),
                    url: "https://api.anthropic.com/v1/messages".to_string(),
                    reachable: true,
                    status_code: Some(400),
                    latency_ms: Some(198),
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
                    status_code: Some(200),
                    latency_ms: Some(183),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "OpenAI API".to_string(),
                    url: "https://api.openai.com/v1/models".to_string(),
                    reachable: true,
                    status_code: Some(401),
                    latency_ms: Some(221),
                    error_type: Some("MOCK_DATA".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: false,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("timeout".to_string()),
                },
                AiServiceResult {
                    name: "Anthropic API".to_string(),
                    url: "https://api.anthropic.com/v1/messages".to_string(),
                    reachable: true,
                    status_code: Some(400),
                    latency_ms: Some(198),
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
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("connection reset".to_string()),
                },
                AiServiceResult {
                    name: "OpenAI API".to_string(),
                    url: "https://api.openai.com/v1/models".to_string(),
                    reachable: false,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("timeout".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: false,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("DNS failed".to_string()),
                },
                AiServiceResult {
                    name: "Anthropic API".to_string(),
                    url: "https://api.anthropic.com/v1/messages".to_string(),
                    reachable: false,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("connection reset".to_string()),
                },
            ],
            local_proxy: "Disabled".to_string(),
            tun_adapters: vec![],
            dns_servers: vec!["223.5.5.5".to_string()],
            gateways: vec!["192.168.1.1".to_string()],
            warnings: vec![],
            errors: vec![
                "All AI services are unreachable".to_string(),
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
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("unknown".to_string()),
                },
                AiServiceResult {
                    name: "OpenAI API".to_string(),
                    url: "https://api.openai.com/v1/models".to_string(),
                    reachable: false,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("unknown".to_string()),
                },
                AiServiceResult {
                    name: "Claude".to_string(),
                    url: "https://claude.ai".to_string(),
                    reachable: false,
                    status_code: None,
                    latency_ms: None,
                    error_type: Some("unknown".to_string()),
                },
                AiServiceResult {
                    name: "Anthropic API".to_string(),
                    url: "https://api.anthropic.com/v1/messages".to_string(),
                    reachable: false,
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

    // Evaluate AI services
    let mut chatgpt_reachable = false;
    let mut openai_reachable = false;
    let mut claude_reachable = false;
    let mut anthropic_reachable = false;

    // Check ChatGPT
    if let Some(chat) = ai_services.iter().find(|a| a.name == "ChatGPT") {
        chatgpt_reachable = chat.reachable;
        if !chat.reachable {
            let err_detail = chat.error_type.as_deref().unwrap_or("unknown");
            warnings.push(format!("ChatGPT is unreachable: {}", err_detail));
        } else if let Some(code) = chat.status_code {
            if code != 200 && code != 301 && code != 302 {
                warnings.push(format!(
                    "ChatGPT returned HTTP {}; network is reachable but response is abnormal",
                    code
                ));
            }
        }
    }

    // Check OpenAI API
    if let Some(oa) = ai_services.iter().find(|a| a.name == "OpenAI API") {
        openai_reachable = oa.reachable;
        if !oa.reachable {
            let err_detail = oa.error_type.as_deref().unwrap_or("unknown");
            warnings.push(format!("OpenAI API is unreachable: {}", err_detail));
        } else if let Some(code) = oa.status_code {
            if code == 404 || code >= 500 {
                warnings.push(format!(
                    "OpenAI API returned HTTP {}; network is reachable but response is abnormal",
                    code
                ));
            }
        }
    }

    // Check Claude & Anthropic API
    if let Some(cl) = ai_services.iter().find(|a| a.name == "Claude") {
        claude_reachable = cl.reachable;
    }
    if let Some(ant) = ai_services.iter().find(|a| a.name == "Anthropic API") {
        anthropic_reachable = ant.reachable;
    }

    // Handle pair-wise blocking rules
    if !chatgpt_reachable && !openai_reachable {
        errors.push(
            "Both ChatGPT and OpenAI API are unreachable (OpenAI ecosystem blocked)".to_string(),
        );
    }
    if !claude_reachable && !anthropic_reachable {
        errors.push(
            "Both Claude and Anthropic API are unreachable (Anthropic ecosystem blocked)"
                .to_string(),
        );
    } else {
        if !claude_reachable {
            let err_detail = ai_services
                .iter()
                .find(|a| a.name == "Claude")
                .and_then(|a| a.error_type.as_deref())
                .unwrap_or("unknown");
            warnings.push(format!("Claude is unreachable: {}", err_detail));
        } else if let Some(cl) = ai_services.iter().find(|a| a.name == "Claude") {
            if let Some(code) = cl.status_code {
                if code != 200 && code != 301 && code != 302 {
                    warnings.push(format!(
                        "Claude returned HTTP {}; network is reachable but response is abnormal",
                        code
                    ));
                }
            }
        }

        if !anthropic_reachable {
            let err_detail = ai_services
                .iter()
                .find(|a| a.name == "Anthropic API")
                .and_then(|a| a.error_type.as_deref())
                .unwrap_or("unknown");
            warnings.push(format!("Anthropic API is unreachable: {}", err_detail));
        } else if let Some(ant) = ai_services.iter().find(|a| a.name == "Anthropic API") {
            if let Some(code) = ant.status_code {
                if code == 404 || code >= 500 {
                    warnings.push(format!("Anthropic API returned HTTP {}; network is reachable but endpoint may be invalid", code));
                }
            }
        }
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

        if ipv4_non_cn
            && ipv6_non_cn
            && chatgpt_reachable
            && openai_reachable
            && (claude_reachable || anthropic_reachable)
        {
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
        if let Some(ref err) = ai.error_type {
            if err.contains("Stage 4 未实现") {
                text.push_str(&format!("- {}：未检测（Stage 4 未实现）\n", ai.name));
                continue;
            }
        }

        let mut parts = Vec::new();
        let label = get_ai_status_label(ai.reachable, ai.status_code);
        parts.push(label);

        if let Some(code) = ai.status_code {
            parts.push(format!("HTTP {}", code));
        }
        if let Some(lat) = ai.latency_ms {
            parts.push(format!("{}ms", lat));
        }
        if !ai.reachable {
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

    text
}
