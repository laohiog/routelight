use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone)]
pub struct IpWhoIsConnection {
    pub asn: Option<u32>,
    pub isp: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct IpWhoIsResponse {
    pub success: bool,
    pub ip: String,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub connection: Option<IpWhoIsConnection>,
}

#[derive(Debug, Clone)]
pub enum Ipv6Result {
    NotDetected,
    QueryFailed(String),
    Success(String),
}

#[derive(Debug, Clone)]
pub struct IpProbeResult {
    pub ipv4: String,
    pub ipv4_geo: Option<IpWhoIsResponse>,
    pub ipv6: Ipv6Result,
    pub ipv6_geo: Option<IpWhoIsResponse>,
    pub warnings: Vec<String>,
}

// Fetch helper for Geolocation (Default: ipwho.is, Fallback: ipinfo.io)
async fn fetch_geo(client: &reqwest::Client, ip: &str) -> Option<IpWhoIsResponse> {
    // 1. Try ipwho.is
    let url = format!("https://ipwho.is/{}", ip);
    if let Ok(resp) = client.get(&url).send().await {
        if let Ok(geo) = resp.json::<IpWhoIsResponse>().await {
            if geo.success {
                return Some(geo);
            }
        }
    }

    // 2. Try ipinfo.io as fallback
    let fallback_url = format!("https://ipinfo.io/{}/json", ip);
    if let Ok(resp) = client.get(&fallback_url).send().await {
        #[derive(Deserialize, Debug, Clone)]
        struct IpInfoResponse {
            pub ip: String,
            pub country: Option<String>,
            pub region: Option<String>,
            pub city: Option<String>,
            pub org: Option<String>,
        }
        if let Ok(info) = resp.json::<IpInfoResponse>().await {
            let (asn, isp) = if let Some(ref org) = info.org {
                // E.g. "AS13335 Cloudflare, Inc."
                let parts: Vec<&str> = org.splitn(2, ' ').collect();
                let asn_parsed = parts.first().and_then(|p| {
                    if let Some(stripped) = p.strip_prefix("AS") {
                        stripped.parse::<u32>().ok()
                    } else {
                        p.parse::<u32>().ok()
                    }
                });
                let isp_parsed = parts.get(1).map(|s| s.to_string());
                (asn_parsed, isp_parsed)
            } else {
                (None, None)
            };

            return Some(IpWhoIsResponse {
                success: true,
                ip: info.ip,
                country_code: info.country,
                region: info.region,
                city: info.city,
                connection: Some(IpWhoIsConnection { asn, isp }),
            });
        }
    }

    None
}

// Parse Cloudflare cdn-cgi/trace to find IP
fn parse_cf_trace(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(ip) = line.strip_prefix("ip=") {
            return Some(ip.to_string());
        }
    }
    None
}

pub async fn probe_ips() -> IpProbeResult {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let mut warnings = Vec::new();

    // 1. Fetch IPv4 Exit IP
    let mut ipv4 = "Detection failed".to_string();
    let mut ipv4_geo = None;

    // Try api.ipify.org
    match client.get("https://api.ipify.org?format=json").send().await {
        Ok(resp) => {
            #[derive(Deserialize)]
            struct IpifyResponse {
                ip: String,
            }
            if let Ok(json) = resp.json::<IpifyResponse>().await {
                ipv4 = json.ip;
            }
        }
        Err(_) => {
            // Fallback to Cloudflare trace
            if let Ok(resp) = client
                .get("https://www.cloudflare.com/cdn-cgi/trace")
                .send()
                .await
            {
                if let Ok(text) = resp.text().await {
                    if let Some(ip) = parse_cf_trace(&text) {
                        ipv4 = ip;
                    }
                }
            }
        }
    }

    if ipv4 != "Detection failed" {
        ipv4_geo = fetch_geo(&client, &ipv4).await;
        if ipv4_geo.is_none() {
            warnings.push("IPv4 geolocation query failed".to_string());
        }
    }

    // 2. Fetch IPv6 Exit IP
    let mut ipv6_geo = None;

    // Probe api6.ipify.org (IPv6 only)
    let ipv6 = match client
        .get("https://api6.ipify.org?format=json")
        .send()
        .await
    {
        Ok(resp) => {
            #[derive(Deserialize)]
            struct IpifyResponse {
                ip: String,
            }
            if let Ok(json) = resp.json::<IpifyResponse>().await {
                let ip_str = json.ip;
                if ip_str.contains(':') {
                    Ipv6Result::Success(ip_str)
                } else {
                    Ipv6Result::NotDetected
                }
            } else {
                Ipv6Result::QueryFailed("Invalid IPv6 response format".to_string())
            }
        }
        Err(err) => {
            // Distinguish between connect failure (no IPv6 support) and other query errors
            if err.is_connect() || err.is_timeout() && err.to_string().contains("dns") {
                Ipv6Result::NotDetected
            } else {
                Ipv6Result::QueryFailed(err.to_string())
            }
        }
    };

    // If IPv6 succeeded, fetch its geo information to check for leakage
    if let Ipv6Result::Success(ref ipv6_addr) = ipv6 {
        ipv6_geo = fetch_geo(&client, ipv6_addr).await;
        if ipv6_geo.is_none() {
            warnings.push("IPv6 geolocation query failed".to_string());
        }
    }

    // Check for IPv6 Direct-Connect risk:
    // If IPv4 is proxied (non-CN) but IPv6 is domestic (CN), raise warning
    if let Some(ref v4) = ipv4_geo {
        if let Some(ref v4_cc) = v4.country_code {
            if v4_cc != "CN" {
                if let Some(ref v6) = ipv6_geo {
                    if let Some(ref v6_cc) = v6.country_code {
                        if v6_cc == "CN" {
                            warnings.push("IPv6 direct-connect risk (IPv6 exit country matches CN while IPv4 is proxied)".to_string());
                        }
                    }
                }
            }
        }
    }

    IpProbeResult {
        ipv4,
        ipv4_geo,
        ipv6,
        ipv6_geo,
        warnings,
    }
}
