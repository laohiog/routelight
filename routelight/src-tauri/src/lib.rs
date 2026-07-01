mod app_state;
mod probe;

use app_state::{
    copy_diagnostics_data, get_cached_status_data, get_status_data, OverallStatus, RouteStatus,
};
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

const AUTO_REFRESH_INTERVAL_SECONDS: u64 = 60;

static LAST_NOTIFIED_STATUS: Mutex<Option<OverallStatus>> = Mutex::new(None);
static LAST_NOTIFIED_IP: Mutex<Option<String>> = Mutex::new(None);

fn update_tray(app: &tauri::AppHandle, status: &RouteStatus) {
    if let Some(tray) = app.tray_by_id("default") {
        let icon_bytes = match status.overall {
            OverallStatus::Normal => include_bytes!("../icons/green.ico").as_slice(),
            OverallStatus::Warning => include_bytes!("../icons/yellow.ico").as_slice(),
            OverallStatus::Error => include_bytes!("../icons/red.ico").as_slice(),
            OverallStatus::Unknown => include_bytes!("../icons/gray.ico").as_slice(),
        };
        let icon = match Image::from_bytes(icon_bytes) {
            Ok(icon) => icon,
            Err(err) => {
                let _ = app.emit("debug-log", format!("[tray] load icon failed: {err}"));
                match Image::from_bytes(include_bytes!("../icons/gray.ico").as_slice()) {
                    Ok(icon) => icon,
                    Err(fallback_err) => {
                        let _ = app.emit(
                            "debug-log",
                            format!("[tray] load fallback icon failed: {fallback_err}"),
                        );
                        return;
                    }
                }
            }
        };

        match tray.set_icon(Some(icon)) {
            Ok(()) => println!("[tray] icon updated to status: {:?}", status.overall),
            Err(err) => {
                let _ = app.emit("debug-log", format!("[tray] set_icon failed: {err}"));
            }
        }
    }
}

fn send_notification(app: &tauri::AppHandle, title: &str, body: &str) {
    println!("[notification] attempt to notify: {} - {}", title, body);

    // Emit notification copy to frontend logs for UI Debug log fallback
    let log_msg = format!("[notification] {}: {}", title, body);
    let _ = app.emit("debug-log", log_msg);

    // Call tauri-plugin-notification safely
    use tauri_plugin_notification::NotificationExt;

    let app_handle = app.clone();
    let title_str = title.to_string();
    let body_str = body.to_string();

    tauri::async_runtime::spawn(async move {
        if let Err(err) = app_handle.notification().request_permission() {
            let _ = app_handle.emit(
                "debug-log",
                format!("[notification] permission request failed: {err}"),
            );
        }
        if let Err(err) = app_handle
            .notification()
            .builder()
            .title(title_str)
            .body(body_str)
            .show()
        {
            let _ = app_handle.emit("debug-log", format!("[notification] send failed: {err}"));
        }
    });
}

fn update_tray_and_notify(app: &tauri::AppHandle, status: &RouteStatus) {
    // 1. Update tray icon
    update_tray(app, status);

    // 2. Lock and Check notifications
    let mut last_status_lock = LAST_NOTIFIED_STATUS.lock().unwrap();
    let mut last_ip_lock = LAST_NOTIFIED_IP.lock().unwrap();

    let old_status = last_status_lock.clone();
    let old_ip = last_ip_lock.clone();

    // 2.1 IP change detection
    if let Some(ref prev_ip) = old_ip {
        if prev_ip != &status.ipv4 && status.ipv4 != "Detection failed" && !status.ipv4.is_empty() {
            let title = "RouteLight 出口已变化";
            let body = format!(
                "{} -> {}，{}，{}",
                prev_ip, status.ipv4, status.country, status.asn
            );
            send_notification(app, title, body.as_str());
        }
    }
    // Update cached IP if it's valid
    if status.ipv4 != "Detection failed" && !status.ipv4.is_empty() {
        *last_ip_lock = Some(status.ipv4.clone());
    }

    // 2.2 Status change detection
    if old_status.as_ref() != Some(&status.overall) {
        let new_status = &status.overall;

        match (&old_status, new_status) {
            // Restore to Normal
            (Some(OverallStatus::Warning) | Some(OverallStatus::Error), OverallStatus::Normal) => {
                let title = "RouteLight 已恢复正常";
                let body = format!("当前出口：{}，{}", status.country, status.ipv4);
                send_notification(app, title, body.as_str());
            }
            // Transition to Error
            (_, OverallStatus::Error) => {
                let has_openai_block = status
                    .errors
                    .iter()
                    .any(|e| e.contains("OpenAI") || e.contains("ChatGPT"));
                let has_ipv6_risk = status.errors.iter().any(|e| e.contains("IPv6"));

                if has_openai_block {
                    let title = "OpenAI 生态可能不可达";
                    let body = status
                        .errors
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "ChatGPT / OpenAI API timeout".to_string());
                    send_notification(app, title, body.as_str());
                } else if has_ipv6_risk {
                    let title = "检测到 IPv6 风险";
                    let body = "IPv6 出口疑似 CN 直连";
                    send_notification(app, title, body);
                } else {
                    let title = "RouteLight 状态异常";
                    let body = status
                        .errors
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "未知错误".to_string());
                    send_notification(app, title, body.as_str());
                }
            }
            // Transition to Warning from Normal, Unknown or initial (first time)
            (
                Some(OverallStatus::Normal) | Some(OverallStatus::Unknown) | None,
                OverallStatus::Warning,
            ) => {
                let title = "RouteLight 状态变为 Warning";
                let body = status
                    .warnings
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "未知警告".to_string());
                send_notification(app, title, body.as_str());
            }
            _ => {}
        }
        *last_status_lock = Some(status.overall.clone());
    }
}

#[tauri::command]
async fn get_status(app: tauri::AppHandle) -> RouteStatus {
    let status = get_status_data().await;
    update_tray_and_notify(&app, &status);
    status
}

#[tauri::command]
async fn refresh_status(app: tauri::AppHandle) -> RouteStatus {
    println!("[menu] refresh");
    let status = get_status_data().await;
    update_tray_and_notify(&app, &status);
    status
}

#[tauri::command]
async fn copy_diagnostics() -> Result<String, String> {
    copy_diagnostics_data().await
}

#[tauri::command]
async fn get_cached_status() -> Option<RouteStatus> {
    get_cached_status_data()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // 1. Load the initial status (blocking call since setup is sync)
            let status = tauri::async_runtime::block_on(async { get_status_data().await });
            update_tray_and_notify(app.handle(), &status);

            let icon_bytes = match status.overall {
                OverallStatus::Normal => include_bytes!("../icons/green.ico").as_slice(),
                OverallStatus::Warning => include_bytes!("../icons/yellow.ico").as_slice(),
                OverallStatus::Error => include_bytes!("../icons/red.ico").as_slice(),
                OverallStatus::Unknown => include_bytes!("../icons/gray.ico").as_slice(),
            };
            let icon = Image::from_bytes(icon_bytes).expect("Failed to load icon");

            // 2. Create menu items
            let open_item = MenuItemBuilder::with_id("open", "Open / 打开").build(app)?;
            let refresh_item = MenuItemBuilder::with_id("refresh", "Refresh / 刷新").build(app)?;
            let copy_item =
                MenuItemBuilder::with_id("copy", "Copy Diagnostics / 复制诊断信息").build(app)?;
            let exit_item = MenuItemBuilder::with_id("exit", "Exit / 退出").build(app)?;

            // 3. Build the menu
            let menu = MenuBuilder::new(app)
                .items(&[&open_item, &refresh_item, &copy_item, &exit_item])
                .build()?;

            // 4. Build the tray icon
            let _tray = TrayIconBuilder::with_id("default")
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button,
                        button_state,
                        ..
                    } = event
                    {
                        match button {
                            MouseButton::Left => {
                                if button_state == MouseButtonState::Up {
                                    println!("[tray] left click");
                                    let app = tray.app_handle();
                                    if let Some(window) = app.get_webview_window("main") {
                                        if window.is_visible().unwrap_or(false) {
                                            println!("[window] hide");
                                            let _ = window.hide();
                                        } else {
                                            println!("[window] show");
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                        }
                                    }
                                }
                            }
                            _ => {
                                println!("[tray] right click ignored");
                            }
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        println!("[menu] open");
                        if let Some(window) = app.get_webview_window("main") {
                            println!("[window] show");
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "refresh" => {
                        println!("[menu] refresh");
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let status = get_status_data().await;
                            update_tray_and_notify(&app_clone, &status);
                            let _ = app_clone.emit("status-refreshed", status);
                        });
                    }
                    "copy" => {
                        println!("[menu] copy diagnostics");
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if copy_diagnostics_data().await.is_ok() {
                                let _ = app_clone.emit("copy-success", ());
                            }
                        });
                    }
                    "exit" => {
                        println!("[menu] exit");
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // 5. Spawn background auto refresh loop
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(
                        AUTO_REFRESH_INTERVAL_SECONDS,
                    ))
                    .await;
                    println!("[monitor] background refresh starting");
                    let status = get_status_data().await;
                    update_tray_and_notify(&app_handle, &status);
                    let _ = app_handle.emit("status-refreshed", status);
                    let _ = app_handle.emit(
                        "debug-log",
                        "[monitor] background refresh completed".to_string(),
                    );
                    println!("[monitor] background refresh completed");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            refresh_status,
            get_cached_status,
            copy_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
