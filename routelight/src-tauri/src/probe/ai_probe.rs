use crate::app_state::AiServiceResult;
use std::time::Duration;

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

async fn check_service(client: &reqwest::Client, name: String, url: String) -> AiServiceResult {
    let start = std::time::Instant::now();
    let request_builder = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

    match request_builder.send().await {
        Ok(resp) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let status_code = resp.status().as_u16();

            // Any HTTP response means the network path is reachable
            let reachable = true;

            AiServiceResult {
                name,
                url,
                reachable,
                status_code: Some(status_code),
                latency_ms: Some(latency_ms),
                error_type: None,
            }
        }
        Err(err) => {
            let error_type = classify_reqwest_error(&err);
            AiServiceResult {
                name,
                url,
                reachable: false,
                status_code: None,
                latency_ms: None,
                error_type: Some(error_type),
            }
        }
    }
}

pub async fn probe_ai_services() -> Vec<AiServiceResult> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .unwrap();

    let services = vec![
        ("ChatGPT".to_string(), "https://chatgpt.com".to_string()),
        (
            "OpenAI API".to_string(),
            "https://api.openai.com/v1/models".to_string(),
        ),
        ("Claude".to_string(), "https://claude.ai".to_string()),
        (
            "Anthropic API".to_string(),
            "https://api.anthropic.com/v1/messages".to_string(),
        ),
    ];

    let mut handles = Vec::new();
    for (name, url) in services {
        let client_clone = client.clone();
        let handle =
            tauri::async_runtime::spawn(
                async move { check_service(&client_clone, name, url).await },
            );
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(res) = handle.await {
            results.push(res);
        }
    }

    results
}
