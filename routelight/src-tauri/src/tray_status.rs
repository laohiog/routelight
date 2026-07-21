use crate::app_state::{AiProbeStatus, AiServiceResult, OverallStatus, RouteStatus};

pub(crate) const TRAY_ICON_SIZE: u32 = 32;

const SERVICE_NAMES: [&str; 3] = ["ChatGPT", "Claude", "Google AI"];
const LIGHT_CENTER_Y: [f32; 3] = [6.0, 16.0, 26.0];
const LIGHT_SEGMENT_START_X: f32 = 6.0;
const LIGHT_SEGMENT_END_X: f32 = 26.0;
const LIGHT_RADIUS: f32 = 4.0;

const GREEN: [u8; 4] = [23, 201, 100, 255];
const YELLOW: [u8; 4] = [255, 193, 7, 255];
const RED: [u8; 4] = [244, 67, 54, 255];
const GRAY: [u8; 4] = [158, 158, 158, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrayLightState {
    Green,
    Yellow,
    Red,
    Gray,
}

impl TrayLightState {
    fn color(self) -> [u8; 4] {
        match self {
            Self::Green => GREEN,
            Self::Yellow => YELLOW,
            Self::Red => RED,
            Self::Gray => GRAY,
        }
    }
}

fn service_by_name<'a>(status: &'a RouteStatus, name: &str) -> Option<&'a AiServiceResult> {
    status
        .ai_services
        .iter()
        .find(|service| service.name == name)
}

fn is_success_status(status_code: Option<u16>) -> bool {
    matches!(status_code, Some(200 | 301 | 302))
}

fn light_state(service: Option<&AiServiceResult>) -> TrayLightState {
    let Some(service) = service else {
        return TrayLightState::Gray;
    };

    match service.probe_status {
        AiProbeStatus::Available => TrayLightState::Green,
        AiProbeStatus::Reachable if is_success_status(service.status_code) => TrayLightState::Green,
        AiProbeStatus::Reachable | AiProbeStatus::ManualCheck => TrayLightState::Yellow,
        AiProbeStatus::RegionRestricted | AiProbeStatus::Unreachable => TrayLightState::Red,
        AiProbeStatus::Unknown => TrayLightState::Gray,
    }
}

fn tray_light_states(status: &RouteStatus) -> [TrayLightState; 3] {
    SERVICE_NAMES.map(|name| light_state(service_by_name(status, name)))
}

fn service_status_label(service: Option<&AiServiceResult>) -> &'static str {
    let Some(service) = service else {
        return "未知";
    };

    match service.probe_status {
        AiProbeStatus::Available => "可用",
        AiProbeStatus::Reachable if is_success_status(service.status_code) => "可达",
        AiProbeStatus::Reachable => "响应异常",
        AiProbeStatus::RegionRestricted => "地区不支持",
        AiProbeStatus::ManualCheck => "需确认",
        AiProbeStatus::Unreachable => "不可达",
        AiProbeStatus::Unknown => "未知",
    }
}

fn overall_status_label(overall: &OverallStatus) -> &'static str {
    match overall {
        OverallStatus::Normal => "正常",
        OverallStatus::Warning => "警告",
        OverallStatus::Error => "异常",
        OverallStatus::Unknown => "未知",
    }
}

pub(crate) fn render_tray_icon(status: &RouteStatus) -> Vec<u8> {
    let mut rgba = vec![0; (TRAY_ICON_SIZE * TRAY_ICON_SIZE * 4) as usize];

    for (center_y, state) in LIGHT_CENTER_Y.into_iter().zip(tray_light_states(status)) {
        let color = state.color();

        for y in 0..TRAY_ICON_SIZE {
            for x in 0..TRAY_ICON_SIZE {
                let pixel_x = x as f32 + 0.5;
                let pixel_y = y as f32 + 0.5;
                let nearest_x = pixel_x.clamp(LIGHT_SEGMENT_START_X, LIGHT_SEGMENT_END_X);
                let delta_x = pixel_x - nearest_x;
                let delta_y = pixel_y - center_y;
                if delta_x * delta_x + delta_y * delta_y <= LIGHT_RADIUS * LIGHT_RADIUS {
                    let offset = ((y * TRAY_ICON_SIZE + x) * 4) as usize;
                    rgba[offset..offset + 4].copy_from_slice(&color);
                }
            }
        }
    }

    rgba
}

pub(crate) fn tray_tooltip(status: &RouteStatus) -> String {
    let labels = SERVICE_NAMES.map(|name| service_status_label(service_by_name(status, name)));

    format!(
        "RouteLight | ChatGPT：{} | Claude：{} | Google AI：{} | 总体：{}",
        labels[0],
        labels[1],
        labels[2],
        overall_status_label(&status.overall)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(
        name: &str,
        probe_status: AiProbeStatus,
        status_code: Option<u16>,
    ) -> AiServiceResult {
        AiServiceResult {
            name: name.to_string(),
            url: format!("https://{}.example", name.to_lowercase().replace(' ', "-")),
            reachable: !matches!(
                probe_status,
                AiProbeStatus::Unreachable | AiProbeStatus::Unknown
            ),
            probe_status,
            status_code,
            latency_ms: Some(100),
            error_type: None,
        }
    }

    fn route_status(overall: OverallStatus, ai_services: Vec<AiServiceResult>) -> RouteStatus {
        RouteStatus {
            overall,
            checked_at: "2026-07-21 12:00:00".to_string(),
            ipv4: "203.0.113.1".to_string(),
            ipv6: "未检测到 IPv6".to_string(),
            country: "US".to_string(),
            city: "Test City".to_string(),
            asn: "AS64500".to_string(),
            isp: "Test ISP".to_string(),
            ai_services,
            local_proxy: "Disabled".to_string(),
            tun_adapters: vec![],
            dns_servers: vec![],
            gateways: vec![],
            warnings: vec![],
            errors: vec![],
        }
    }

    fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * TRAY_ICON_SIZE + x) * 4) as usize;
        rgba[offset..offset + 4].try_into().unwrap()
    }

    #[test]
    fn maps_probe_states_and_http_codes_to_light_states() {
        assert_eq!(
            light_state(Some(&service(
                "service",
                AiProbeStatus::Available,
                Some(200)
            ))),
            TrayLightState::Green
        );

        for status_code in [Some(200), Some(301), Some(302)] {
            assert_eq!(
                light_state(Some(&service(
                    "service",
                    AiProbeStatus::Reachable,
                    status_code
                ))),
                TrayLightState::Green
            );
        }

        for status_code in [Some(403), Some(500), None] {
            assert_eq!(
                light_state(Some(&service(
                    "service",
                    AiProbeStatus::Reachable,
                    status_code
                ))),
                TrayLightState::Yellow
            );
        }

        assert_eq!(
            light_state(Some(&service(
                "service",
                AiProbeStatus::ManualCheck,
                Some(403)
            ))),
            TrayLightState::Yellow
        );
        assert_eq!(
            light_state(Some(&service(
                "service",
                AiProbeStatus::RegionRestricted,
                Some(200)
            ))),
            TrayLightState::Red
        );
        assert_eq!(
            light_state(Some(&service("service", AiProbeStatus::Unreachable, None))),
            TrayLightState::Red
        );
        assert_eq!(
            light_state(Some(&service("service", AiProbeStatus::Unknown, None))),
            TrayLightState::Gray
        );
        assert_eq!(light_state(None), TrayLightState::Gray);
    }

    #[test]
    fn uses_fixed_name_based_order_and_ignores_extra_services() {
        let status = route_status(
            OverallStatus::Warning,
            vec![
                service("Google AI", AiProbeStatus::RegionRestricted, Some(200)),
                service("Extra", AiProbeStatus::Available, Some(200)),
                service("ChatGPT", AiProbeStatus::Reachable, Some(403)),
                service("Claude", AiProbeStatus::Reachable, Some(200)),
            ],
        );

        assert_eq!(
            tray_light_states(&status),
            [
                TrayLightState::Yellow,
                TrayLightState::Green,
                TrayLightState::Red
            ]
        );

        let missing = route_status(
            OverallStatus::Unknown,
            vec![service("Claude", AiProbeStatus::Reachable, Some(200))],
        );
        assert_eq!(
            tray_light_states(&missing),
            [
                TrayLightState::Gray,
                TrayLightState::Green,
                TrayLightState::Gray
            ]
        );
    }

    #[test]
    fn renders_capsules_with_expected_bounds_gaps_and_colors() {
        let status = route_status(
            OverallStatus::Warning,
            vec![
                service("ChatGPT", AiProbeStatus::Reachable, Some(403)),
                service("Claude", AiProbeStatus::Reachable, Some(200)),
                service("Google AI", AiProbeStatus::RegionRestricted, Some(200)),
            ],
        );
        let rgba = render_tray_icon(&status);

        assert_eq!(rgba.len(), (TRAY_ICON_SIZE * TRAY_ICON_SIZE * 4) as usize);
        assert_eq!(pixel(&rgba, 0, 0), [0, 0, 0, 0]);
        assert_eq!(pixel(&rgba, 16, 6), YELLOW);
        assert_eq!(pixel(&rgba, 16, 16), GREEN);
        assert_eq!(pixel(&rgba, 16, 26), RED);

        assert_eq!(pixel(&rgba, 2, 6), YELLOW);
        assert_eq!(pixel(&rgba, 29, 6), YELLOW);
        assert_eq!(pixel(&rgba, 1, 6), [0, 0, 0, 0]);
        assert_eq!(pixel(&rgba, 30, 6), [0, 0, 0, 0]);
        assert_eq!(pixel(&rgba, 2, 2), [0, 0, 0, 0]);
        assert_eq!(pixel(&rgba, 16, 10), [0, 0, 0, 0]);
        assert_eq!(pixel(&rgba, 16, 11), [0, 0, 0, 0]);
        assert_eq!(pixel(&rgba, 16, 20), [0, 0, 0, 0]);
        assert_eq!(pixel(&rgba, 16, 21), [0, 0, 0, 0]);

        let changed = route_status(
            OverallStatus::Normal,
            vec![
                service("ChatGPT", AiProbeStatus::Reachable, Some(200)),
                service("Claude", AiProbeStatus::Reachable, Some(200)),
                service("Google AI", AiProbeStatus::Available, Some(200)),
            ],
        );
        assert_ne!(rgba, render_tray_icon(&changed));
    }

    #[test]
    fn builds_fixed_order_tooltip_within_windows_limit() {
        let status = route_status(
            OverallStatus::Warning,
            vec![
                service("Google AI", AiProbeStatus::RegionRestricted, Some(200)),
                service("ChatGPT", AiProbeStatus::Reachable, Some(403)),
                service("Claude", AiProbeStatus::Available, Some(200)),
            ],
        );
        let tooltip = tray_tooltip(&status);

        assert_eq!(
            tooltip,
            "RouteLight | ChatGPT：响应异常 | Claude：可用 | Google AI：地区不支持 | 总体：警告"
        );
        assert!(tooltip.encode_utf16().count() < 128);
    }
}
