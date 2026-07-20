use crate::app_state::{AiProbeStatus, AiServiceResult};
use std::time::Duration;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const GOOGLE_AI_URL: &str = "https://www.google.com/ai?hl=en";
const MAX_GOOGLE_BODY_BYTES: usize = 256 * 1024;

fn classify_reqwest_error(err: &reqwest::Error) -> String {
    let err_str = err.to_string().to_lowercase();
    if err.is_timeout() {
        "timeout".to_string()
    } else if err_str.contains("dns") || err_str.contains("resolve") || err_str.contains("lookup") {
        "DNS failed".to_string()
    } else if err_str.contains("ssl")
        || err_str.contains("tls")
        || err_str.contains("schannel")
        || err_str.contains("cert")
        || err_str.contains("handshake")
    {
        "TLS failed".to_string()
    } else if err_str.contains("reset") || err_str.contains("forcibly closed") {
        "connection reset".to_string()
    } else {
        err.to_string()
    }
}

async fn check_service(client: &reqwest::Client, name: &str, url: &str) -> AiServiceResult {
    let start = std::time::Instant::now();
    let request_builder = client.get(url).header("User-Agent", USER_AGENT);

    match request_builder.send().await {
        Ok(resp) => AiServiceResult {
            name: name.to_string(),
            url: url.to_string(),
            reachable: true,
            probe_status: AiProbeStatus::Reachable,
            status_code: Some(resp.status().as_u16()),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error_type: None,
        },
        Err(err) => AiServiceResult {
            name: name.to_string(),
            url: url.to_string(),
            reachable: false,
            probe_status: AiProbeStatus::Unreachable,
            status_code: None,
            latency_ms: None,
            error_type: Some(classify_reqwest_error(&err)),
        },
    }
}

fn is_google_ai_mode_url(final_url: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(final_url) else {
        return false;
    };

    let is_google_host = url
        .host_str()
        .map(|host| host == "google.com" || host.ends_with(".google.com"))
        .unwrap_or(false);

    is_google_host
        && url.path() == "/search"
        && url
            .query_pairs()
            .any(|(key, value)| key == "udm" && value == "50")
}

fn classify_google_ai_response(
    status_code: u16,
    final_url: &str,
    body: &str,
) -> (AiProbeStatus, Option<String>) {
    if status_code < 400 && is_google_ai_mode_url(final_url) {
        return (AiProbeStatus::Available, None);
    }

    let lower_url = final_url.to_ascii_lowercase();
    let lower_body = body.to_ascii_lowercase();

    if lower_body.contains("ai mode is not available in your country or language")
        || lower_body.contains("ai mode isn't available in your country or language")
        || lower_body.contains("ai mode is not available in your country")
    {
        return (
            AiProbeStatus::RegionRestricted,
            Some("region unsupported".to_string()),
        );
    }

    if status_code == 429
        || status_code == 403
        || lower_url.contains("/sorry/")
        || lower_body.contains("unusual traffic")
        || lower_body.contains("recaptcha")
    {
        return (
            AiProbeStatus::ManualCheck,
            Some("Google verification required".to_string()),
        );
    }

    if lower_url.contains("consent.google.") || lower_body.contains("before you continue to google")
    {
        return (
            AiProbeStatus::ManualCheck,
            Some("Google consent required".to_string()),
        );
    }

    if lower_body.contains("ai mode is not currently available on your device or account")
        || lower_body.contains("ai mode is not available for your account")
    {
        return (
            AiProbeStatus::ManualCheck,
            Some("device or account eligibility requires manual verification".to_string()),
        );
    }

    (
        AiProbeStatus::Unknown,
        Some(format!(
            "Google AI availability could not be determined (HTTP {status_code})"
        )),
    )
}

async fn read_response_body_limited(
    mut response: reqwest::Response,
) -> Result<String, reqwest::Error> {
    let mut bytes = Vec::new();

    while let Some(chunk) = response.chunk().await? {
        let remaining = MAX_GOOGLE_BODY_BYTES.saturating_sub(bytes.len());
        if remaining == 0 {
            break;
        }

        let take = remaining.min(chunk.len());
        bytes.extend_from_slice(&chunk[..take]);
        if bytes.len() == MAX_GOOGLE_BODY_BYTES {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

async fn check_google_ai(client: &reqwest::Client) -> AiServiceResult {
    let start = std::time::Instant::now();
    let request = client
        .get(GOOGLE_AI_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept-Language", "en-US,en;q=0.9")
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        );

    match request.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let final_url = response.url().as_str().to_string();

            let (mut probe_status, mut error_type) =
                classify_google_ai_response(status_code, &final_url, "");

            if probe_status != AiProbeStatus::Available {
                match read_response_body_limited(response).await {
                    Ok(body) => {
                        (probe_status, error_type) =
                            classify_google_ai_response(status_code, &final_url, &body);
                    }
                    Err(err) => {
                        probe_status = AiProbeStatus::Unknown;
                        error_type = Some(format!(
                            "Google AI response read failed: {}",
                            classify_reqwest_error(&err)
                        ));
                    }
                }
            }

            AiServiceResult {
                name: "Google AI".to_string(),
                url: GOOGLE_AI_URL.to_string(),
                reachable: true,
                probe_status,
                status_code: Some(status_code),
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error_type,
            }
        }
        Err(err) => AiServiceResult {
            name: "Google AI".to_string(),
            url: GOOGLE_AI_URL.to_string(),
            reachable: false,
            probe_status: AiProbeStatus::Unreachable,
            status_code: None,
            latency_ms: None,
            error_type: Some(classify_reqwest_error(&err)),
        },
    }
}

pub async fn probe_ai_services() -> Vec<AiServiceResult> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap();

    let chatgpt = check_service(&client, "ChatGPT", "https://chatgpt.com");
    let claude = check_service(&client, "Claude", "https://claude.ai");
    let google_ai = check_google_ai(&client);
    let (chatgpt, claude, google_ai) = tokio::join!(chatgpt, claude, google_ai);

    vec![chatgpt, claude, google_ai]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_google_ai_mode_redirect() {
        let (status, error) =
            classify_google_ai_response(200, "https://www.google.com/search?udm=50&aep=11", "");

        assert_eq!(status, AiProbeStatus::Available);
        assert_eq!(error, None);
    }

    #[test]
    fn recognizes_explicit_region_restriction() {
        let (status, error) = classify_google_ai_response(
            200,
            "https://www.google.com/ai?hl=en",
            "AI Mode is not available in your country or language",
        );

        assert_eq!(status, AiProbeStatus::RegionRestricted);
        assert_eq!(error.as_deref(), Some("region unsupported"));
    }

    #[test]
    fn treats_google_challenge_as_manual_check() {
        let (status, error) = classify_google_ai_response(
            429,
            "https://www.google.com/sorry/index",
            "Our systems have detected unusual traffic",
        );

        assert_eq!(status, AiProbeStatus::ManualCheck);
        assert_eq!(error.as_deref(), Some("Google verification required"));
    }

    #[test]
    fn does_not_misclassify_ambiguous_page_as_unavailable() {
        let (status, error) =
            classify_google_ai_response(200, "https://www.google.com/ai?hl=en", "Google Search");

        assert_eq!(status, AiProbeStatus::Unknown);
        assert!(error
            .as_deref()
            .unwrap_or_default()
            .contains("could not be determined"));
    }
}
