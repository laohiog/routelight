# Repository Guidelines

## Project Structure & Module Organization

This repository contains the RouteLight desktop app. The main app lives in `routelight/`; run most commands from that directory. Frontend files are in `routelight/src/` (`index.html`, `main.js`, `styles.css`). Tauri/Rust backend code is in `routelight/src-tauri/src/`, with probe logic under `src-tauri/src/probe/`. Tauri configuration is `routelight/src-tauri/tauri.conf.json`, Rust dependencies are in `src-tauri/Cargo.toml`, app icons are in `src-tauri/icons/`, and public design notes live under `docs/`.

## Build, Test, and Development Commands

Run these from `routelight/` unless noted:

- `npm install` installs the Tauri CLI and frontend dependencies.
- `npm run tauri dev` starts the Windows Tauri app in development mode.
- `$env:ROUTELIGHT_MOCK_STATUS="normal"; npm run tauri dev` starts mock status rendering; valid examples include `normal`, `warning`, and `error`.
- `npm run tauri build` creates Windows release artifacts under `src-tauri/target/release/`.
- `cargo test` from `routelight/src-tauri/` runs Rust tests when tests are added.

## Coding Style & Naming Conventions

Use two-space indentation in JavaScript/CSS and standard `rustfmt` formatting for Rust. Keep frontend code as plain ES modules and interact with backend commands through `window.__TAURI__`. Use `camelCase` for JavaScript variables/functions, `snake_case` for Rust functions/modules, and descriptive Tauri command names such as `refresh_status`. Keep user-visible Chinese/English UI labels consistent with the existing bilingual style.

## Testing Guidelines

There is no committed automated test suite yet. For Rust changes, add focused unit tests near the relevant module and run `cargo test`. For frontend or tray behavior, verify manually with `npm run tauri dev` and the mock status environment variable. Network-probe changes should document which live endpoints were exercised and whether mock mode was used.

## Commit & Pull Request Guidelines

Use concise Conventional Commit-style messages, for example `feat: add IPv6 warning probe` or `fix: avoid duplicate refresh notification`. Pull requests should include a short summary, verification steps, linked issue or task context, and screenshots/GIFs for panel or tray UI changes. Call out any new network endpoint, permission, or Windows-specific behavior.

## Security & Configuration Tips

RouteLight must remain non-intrusive: do not add proxy mutation, route-table changes, packet capture, credential access, clipboard reads, or shell-command execution. Keep diagnostics in memory unless a user explicitly copies them. Never commit secrets, local `.env` files, build outputs, or `src-tauri/target/`.
