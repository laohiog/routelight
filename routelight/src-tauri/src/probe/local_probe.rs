use winreg::enums::*;
use winreg::RegKey;

#[derive(Clone, Debug)]
pub struct LocalNetworkInfo {
    pub proxy: String,
    pub tun_adapters: Vec<String>,
    pub dns_servers: Vec<String>,
    pub gateways: Vec<String>,
}

const SUSPECTED_TUN_KEYWORDS: [&str; 13] = [
    "tun",
    "tap",
    "wintun",
    "clash",
    "mihomo",
    "meta",
    "v2ray",
    "sing-box",
    "vpn",
    "openvpn",
    "wireguard",
    "tailscale",
    "zerotier",
];

fn is_suspected_tun_adapter(friendly_name: &str, description: &str) -> bool {
    let friendly_name = friendly_name.to_lowercase();
    let description = description.to_lowercase();

    SUSPECTED_TUN_KEYWORDS
        .iter()
        .any(|keyword| friendly_name.contains(keyword) || description.contains(keyword))
}

fn format_system_proxy(proxy_enable: u32, proxy_server: &str, auto_config_url: &str) -> String {
    let manual_proxy_enabled = proxy_enable == 1 && !proxy_server.is_empty();

    if manual_proxy_enabled && !auto_config_url.is_empty() {
        format!("Manual: {} | PAC: {}", proxy_server, auto_config_url)
    } else if manual_proxy_enabled {
        proxy_server.to_string()
    } else if !auto_config_url.is_empty() {
        format!("PAC: {}", auto_config_url)
    } else {
        "Disabled".to_string()
    }
}

pub fn get_system_proxy() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

    if let Ok(key) = hkcu.open_subkey(subkey) {
        let proxy_enable: u32 = key.get_value("ProxyEnable").unwrap_or(0);
        let proxy_server: String = key.get_value("ProxyServer").unwrap_or_default();
        let auto_config_url: String = key.get_value("AutoConfigURL").unwrap_or_default();

        format_system_proxy(proxy_enable, &proxy_server, &auto_config_url)
    } else {
        "Disabled".to_string()
    }
}

pub fn probe_local_network() -> LocalNetworkInfo {
    let proxy = get_system_proxy();
    let mut tun_adapters = Vec::new();
    let mut dns_servers = Vec::new();
    let mut gateways = Vec::new();

    if let Ok(adapters) = ipconfig::get_adapters() {
        for adapter in adapters {
            if adapter.oper_status() == ipconfig::OperStatus::IfOperStatusUp {
                if is_suspected_tun_adapter(adapter.friendly_name(), adapter.description()) {
                    tun_adapters.push(adapter.friendly_name().to_string());
                }

                for dns in adapter.dns_servers() {
                    let dns_str = dns.to_string();
                    if !dns_servers.contains(&dns_str) {
                        dns_servers.push(dns_str);
                    }
                }

                for gw in adapter.gateways() {
                    let gw_str = gw.to_string();
                    if !gateways.contains(&gw_str) {
                        gateways.push(gw_str);
                    }
                }
            }
        }
    }

    LocalNetworkInfo {
        proxy,
        tun_adapters,
        dns_servers,
        gateways,
    }
}

#[cfg(test)]
mod tests {
    use super::{format_system_proxy, is_suspected_tun_adapter};

    #[test]
    fn detects_new_keywords_case_insensitively_in_friendly_names() {
        for keyword in ["MeTa", "V2Ray", "OpenVPN"] {
            assert!(is_suspected_tun_adapter(keyword, ""));
        }
    }

    #[test]
    fn detects_new_keywords_case_insensitively_in_descriptions() {
        for keyword in ["mETA", "V2RAY", "oPeNvPn"] {
            assert!(is_suspected_tun_adapter("Ethernet Adapter", keyword));
        }
    }

    #[test]
    fn still_detects_existing_keyword_and_ignores_ordinary_adapter_text() {
        assert!(is_suspected_tun_adapter("Wintun Adapter", ""));
        assert!(!is_suspected_tun_adapter(
            "Ethernet Adapter",
            "Generic network interface"
        ));
    }

    #[test]
    fn formats_enabled_manual_proxy_and_pac_together() {
        assert_eq!(
            format_system_proxy(1, "127.0.0.1:7890", "https://proxy.example/pac"),
            "Manual: 127.0.0.1:7890 | PAC: https://proxy.example/pac"
        );
    }

    #[test]
    fn preserves_manual_proxy_value_when_pac_is_absent() {
        assert_eq!(
            format_system_proxy(1, "http=proxy:80;https=proxy:443", ""),
            "http=proxy:80;https=proxy:443"
        );
    }

    #[test]
    fn reports_pac_when_manual_proxy_is_not_enabled() {
        assert_eq!(
            format_system_proxy(0, "", "https://proxy.example/pac"),
            "PAC: https://proxy.example/pac"
        );
    }

    #[test]
    fn reports_disabled_when_neither_proxy_source_is_available() {
        assert_eq!(format_system_proxy(0, "", ""), "Disabled");
    }

    #[test]
    fn ignores_stale_manual_proxy_when_manual_proxy_is_not_enabled() {
        assert_eq!(format_system_proxy(0, "stale-proxy:8080", ""), "Disabled");
    }
}
