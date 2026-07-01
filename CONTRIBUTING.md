# Contributing to RouteLight

Thanks for helping improve RouteLight. The project is intentionally small and conservative: it should stay a read-only Windows tray utility for network route diagnostics.

## Setup

Prerequisites:

- Windows 10 or Windows 11
- Node.js 18+
- Rustup with the stable MSVC toolchain
- Microsoft C++ Build Tools
- WebView2 Runtime

Install dependencies from the app directory:

```powershell
cd routelight
npm install
```

## Development

Run the Tauri app:

```powershell
cd routelight
npm run tauri dev
```

Mock UI states:

```powershell
$env:ROUTELIGHT_MOCK_STATUS="normal"; npm run tauri dev
$env:ROUTELIGHT_MOCK_STATUS="warning"; npm run tauri dev
$env:ROUTELIGHT_MOCK_STATUS="error"; npm run tauri dev
```

## Checks Before a PR

Run Rust checks from `routelight/src-tauri/`:

```powershell
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

Run the release build from `routelight/`:

```powershell
npm run tauri build
```

## Pull Request Guidelines

- Keep PRs small and focused.
- Explain user-visible behavior changes.
- Include verification steps and screenshots/GIFs for UI or tray changes.
- Call out new network endpoints, permissions, or Windows-specific behavior.
- Do not include build output, installers, `target/`, `node_modules/`, `.env`, or local logs.

## Security Boundaries

Do not widen RouteLight's non-intrusive behavior without prior discussion.

PRs should not add:

- Proxy mutation, route-table changes, VPN node switching, or proxy subscription management.
- Packet capture or browser cookie/credential access.
- Clipboard reads.
- Diagnostic uploads or diagnostic files written to disk.
- Runtime shell, PowerShell, cmd, or `std::process` execution.
- New Tauri `shell`, `fs`, `opener`, `http`, or clipboard-read capabilities.
- IP purity, fraud, or reputation scoring.
