const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// Dom elements
let statusBadge;
let ipv4Val, ipv6Val, locationVal, ispVal;
let servicesList;
let proxyVal, tunVal;
let alertsContainer;
let timeVal;
let refreshBtn, copyBtn;
let debugArea;
let healthStrip, statusSummary;

const statusCopy = {
  normal: { badge: "● NORMAL", summary: "路由状态正常，AI 服务路径可用" },
  warning: { badge: "▲ WARNING", summary: "检测到风险，请查看警告项" },
  error: { badge: "■ ERROR", summary: "检测到异常，请优先处理错误项" },
  unknown: { badge: "◆ UNKNOWN", summary: "等待检测结果" },
};

function logDebug(msg) {
  console.log(msg);
  if (debugArea) {
    // Prepend to show most recent log at the top
    debugArea.textContent = `${msg}\n${debugArea.textContent}`.substring(0, 1000);
  }
}

async function fetchStatus(isRefresh = false) {
  try {
    if (isRefresh) {
      logDebug("[frontend] refresh button clicked");
      refreshBtn.textContent = "刷新中...";
      refreshBtn.disabled = true;
    }

    const cmd = isRefresh ? "refresh_status" : "get_status";
    logDebug(`[frontend] invoking tauri command: ${cmd}`);

    const status = await invoke(cmd);
    logDebug("[frontend] status received successfully");
    logDebug(`[frontend] status details:\n${JSON.stringify(status, null, 2)}`);
    renderStatus(status);

    if (isRefresh) {
      logDebug("[frontend] refresh completed successfully");
    }
  } catch (err) {
    const errMsg = err.message || JSON.stringify(err) || String(err);
    logDebug(`[frontend] error during fetchStatus: ${errMsg}`);
  } finally {
    if (isRefresh) {
      refreshBtn.textContent = "刷新";
      refreshBtn.disabled = false;
    }
  }
}

async function renderCachedStatus() {
  try {
    logDebug("[frontend] status-refreshed payload missing; reading cached status");
    const cachedStatus = await invoke("get_cached_status");
    if (cachedStatus) {
      renderStatus(cachedStatus);
      logDebug("[frontend] cached status rendered");
    } else {
      logDebug("[frontend] no cached status available");
    }
  } catch (err) {
    const errMsg = err.message || JSON.stringify(err) || String(err);
    logDebug(`[frontend] cached status read failed: ${errMsg}`);
  }
}

function renderStatus(status) {
  // 1. Overall Badge
  const overall = status.overall || "unknown";
  const overallCopy = statusCopy[overall] || statusCopy.unknown;
  statusBadge.className = `badge status-${overall}`;
  statusBadge.textContent = overallCopy.badge;
  healthStrip.className = `health-strip status-${overall}`;
  statusSummary.textContent = overallCopy.summary;

  // 2. Geolocation Info
  ipv4Val.textContent = status.ipv4 || "-";
  ipv6Val.textContent = status.ipv6 || "-";

  if (status.country && status.country !== "Unknown") {
    locationVal.textContent = `${status.country} · ${status.city || ""}`;
  } else {
    locationVal.textContent = "-";
  }

  if (status.isp && status.isp !== "Unknown") {
    ispVal.textContent = `${status.asn ? status.asn + " " : ""}${status.isp}`;
  } else {
    ispVal.textContent = "-";
  }

  // 3. AI Services List
  servicesList.innerHTML = "";
  status.ai_services.forEach(service => {
    const item = document.createElement("div");

    const nameGroup = document.createElement("div");
    nameGroup.className = "service-name-group";
    const nameSpan = document.createElement("span");
    nameSpan.className = "service-name";
    nameSpan.textContent = service.name;
    nameGroup.appendChild(nameSpan);

    const statusGroup = document.createElement("div");
    statusGroup.className = "service-status-group";

    const statusSpan = document.createElement("span");
    statusSpan.className = "service-status";

    let statusText = "";
    let statusClass = "unreachable";

    if (service.error_type && service.error_type.includes("Stage 4 未实现")) {
      statusText = "◆ 未检测";
      statusClass = "unknown";
    } else if (service.reachable) {
      if (service.status_code === 200 || service.status_code === 301 || service.status_code === 302) {
        statusText = "● 可达";
        statusClass = "reachable";
      } else if (service.status_code === 401) {
        statusText = "▲ API 可达但未认证";
        statusClass = "unauthorized";
      } else if (service.status_code === 403) {
        statusText = "▲ 可达但受限";
        statusClass = "unauthorized";
      } else if (service.status_code === 400) {
        statusText = "▲ API 可达但请求无效";
        statusClass = "unauthorized";
      } else if (service.status_code === 404) {
        statusText = "▲ HTTP 可达但端点无效";
        statusClass = "unauthorized";
      } else if (service.status_code === 405) {
        statusText = "▲ HTTP 可达但方法不允许";
        statusClass = "unauthorized";
      } else {
        statusText = "▲ HTTP 可达但响应异常";
        statusClass = "unauthorized";
      }
    } else {
      statusText = "■ 不可达";
      statusClass = "unreachable";
    }

    item.className = `service-item is-${statusClass}`;
    statusSpan.textContent = statusText;
    statusSpan.className = `service-status ${statusClass}`;
    statusGroup.appendChild(statusSpan);

    // Latency or Error detail
    const detailsSpan = document.createElement("span");
    detailsSpan.className = "service-latency";

    if (service.error_type && service.error_type.includes("Stage 4 未实现")) {
      detailsSpan.textContent = " · Stage 4 未实现";
    } else if (service.reachable) {
      const codePart = service.status_code ? ` · ${service.status_code}` : "";
      const latencyPart = service.latency_ms !== null ? ` · ${service.latency_ms}ms` : "";
      detailsSpan.textContent = `${codePart}${latencyPart}`;
    } else if (service.error_type) {
      detailsSpan.textContent = ` · ${service.error_type}`;
    }
    statusGroup.appendChild(detailsSpan);

    item.appendChild(nameGroup);
    item.appendChild(statusGroup);
    servicesList.appendChild(item);
  });

  // 4. Local network status
  proxyVal.textContent = status.local_proxy || "Disabled";
  tunVal.textContent = status.tun_adapters && status.tun_adapters.length > 0
    ? status.tun_adapters.join(", ")
    : "无";

  // 5. Update Time
  timeVal.textContent = status.checked_at || "-";

  // 6. Warnings / Errors
  alertsContainer.innerHTML = "";
  if (status.errors && status.errors.length > 0) {
    alertsContainer.className = "alerts-container has-errors";
    status.errors.forEach(err => {
      const p = document.createElement("div");
      p.textContent = `■ 错误: ${err}`;
      alertsContainer.appendChild(p);
    });
  } else if (status.warnings && status.warnings.length > 0) {
    alertsContainer.className = "alerts-container has-warnings";
    status.warnings.forEach(warn => {
      const p = document.createElement("div");
      p.textContent = `▲ 警告: ${warn}`;
      alertsContainer.appendChild(p);
    });
  } else {
    alertsContainer.className = "alerts-container hidden";
  }
}

async function copyDiagnostics() {
  try {
    logDebug("[frontend] copy diagnostics button clicked");
    copyBtn.textContent = "已复制";
    copyBtn.disabled = true;

    await invoke("copy_diagnostics");
    logDebug("diagnostics copied successfully");

    setTimeout(() => {
      copyBtn.textContent = "复制诊断";
      copyBtn.disabled = false;
    }, 1000);
  } catch (err) {
    const errMsg = err.message || JSON.stringify(err) || String(err);
    logDebug(`[frontend] copy diagnostics error: ${errMsg}`);
    copyBtn.textContent = "复制失败";
    setTimeout(() => {
      copyBtn.textContent = "复制诊断";
      copyBtn.disabled = false;
    }, 1000);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  // Bind Debug Area first
  debugArea = document.querySelector("#debug-area");

  logDebug("[frontend] DOMContentLoaded triggered");
  logDebug("[frontend] main.js successfully loaded");

  // Bind UI Elements
  statusBadge = document.querySelector("#status-badge");
  ipv4Val = document.querySelector("#ipv4-val");
  ipv6Val = document.querySelector("#ipv6-val");
  locationVal = document.querySelector("#location-val");
  ispVal = document.querySelector("#isp-val");
  servicesList = document.querySelector("#services-list");
  proxyVal = document.querySelector("#proxy-val");
  tunVal = document.querySelector("#tun-val");
  alertsContainer = document.querySelector("#alerts-container");
  timeVal = document.querySelector("#time-val");
  refreshBtn = document.querySelector("#refresh-btn");
  copyBtn = document.querySelector("#copy-btn");
  healthStrip = document.querySelector("#health-strip");
  statusSummary = document.querySelector("#status-summary");

  logDebug("[frontend] DOM elements bound successfully");

  // Bind Window Draggability fallback
  const header = document.querySelector(".header");
  header?.addEventListener("mousedown", async (event) => {
    if (event.button !== 0) return;
    if (event.detail > 1) return; // Prevent startDragging on double clicks to avoid flashing

    const interactive = event.target.closest("button, input, textarea, select, a, .badge");
    if (interactive) return;

    try {
      const appWindow = window.__TAURI__?.window?.getCurrentWindow?.();
      if (appWindow?.startDragging) {
        await appWindow.startDragging();
        console.log("[frontend] window drag started");
      } else {
        logDebug("[frontend] startDragging API unavailable");
      }
    } catch (err) {
      logDebug(`[frontend] startDragging failed: ${err}`);
    }
  });

  // Add Event Listeners
  refreshBtn.addEventListener("click", () => fetchStatus(true));
  copyBtn.addEventListener("click", copyDiagnostics);
  logDebug("[frontend] button click handlers registered");

  // Initial Load
  fetchStatus();

  // Listen to menu-triggered refreshes from Rust
  listen("status-refreshed", (event) => {
    logDebug("[frontend] received status-refreshed snapshot from Rust");
    if (event.payload) {
      renderStatus(event.payload);
      logDebug("[frontend] refreshed snapshot rendered without duplicate probe");
    } else {
      renderCachedStatus();
    }
  });

  // Listen to menu-triggered copy from Rust
  listen("copy-success", () => {
    logDebug("diagnostics copied successfully");
  });

  // Listen to debug logs from Rust (notifications fallback)
  listen("debug-log", (event) => {
    logDebug(event.payload);
  });
});
