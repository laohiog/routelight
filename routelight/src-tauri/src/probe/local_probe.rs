use winreg::enums::*;
use winreg::RegKey;

#[derive(Clone, Debug)]
pub struct LocalNetworkInfo {
    pub proxy: String,
    pub tun_adapters: Vec<String>,
    pub dns_servers: Vec<String>,
    pub gateways: Vec<String>,
}

pub fn get_system_proxy() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

    if let Ok(key) = hkcu.open_subkey(subkey) {
        let proxy_enable: u32 = key.get_value("ProxyEnable").unwrap_or(0);
        let proxy_server: String = key.get_value("ProxyServer").unwrap_or_default();
        let auto_config_url: String = key.get_value("AutoConfigURL").unwrap_or_default();

        if proxy_enable == 1 && !proxy_server.is_empty() {
            proxy_server
        } else if !auto_config_url.is_empty() {
            format!("PAC: {}", auto_config_url)
        } else {
            "Disabled".to_string()
        }
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
