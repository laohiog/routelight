use winreg::enums::*;
use winreg::RegKey;

#[derive(Clone, Debug)]
pub struct LocalNetworkInfo {
    pub proxy: String,
    pub tun_adapters: Vec<String>,
    pub dns_servers: Vec<String>,
    pub gateways: Vec<String>,
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
                let name = adapter.friendly_name().to_lowercase();
                let desc = adapter.description().to_lowercase();

                let is_tun = name.contains("tun")
                    || name.contains("tap")
                    || name.contains("wintun")
                    || name.contains("clash")
                    || name.contains("mihomo")
                    || name.contains("sing-box")
                    || name.contains("vpn")
                    || name.contains("wireguard")
                    || name.contains("tailscale")
                    || name.contains("zerotier")
                    || desc.contains("tun")
                    || desc.contains("tap")
                    || desc.contains("wintun")
                    || desc.contains("clash")
                    || desc.contains("mihomo")
                    || desc.contains("sing-box")
                    || desc.contains("vpn")
                    || desc.contains("wireguard")
                    || desc.contains("tailscale")
                    || desc.contains("zerotier");

                if is_tun {
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
    use super::format_system_proxy;

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
