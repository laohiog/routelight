# Repository Guidelines

## Project Structure & Module Organization

This repository contains the RouteLight desktop app. The main app lives in `routelight/`; run most commands from that directory. Frontend files are in `routelight/src/` (`index.html`, `main.js`, `styles.css`). Tauri/Rust backend code is in `routelight/src-tauri/src/`, with probe logic under `src-tauri/src/probe/`. Tauri configuration is `routelight/src-tauri/tauri.conf.json`, Rust dependencies are in `src-tauri/Cargo.toml`, app icons are in `src-tauri/icons/`, and public design notes live under `docs/`.

## Build, Test, and Development Commands

Run these from `routelight/` unless noted:

- `npm ci` installs the frontend and Tauri CLI dependencies reproducibly, as in CI; use `npm install` for local dependency updates.
- `npm run tauri dev` starts the Windows Tauri app in development mode.
- `$env:ROUTELIGHT_MOCK_STATUS="normal"; npm run tauri dev` starts mock status rendering; valid examples include `normal`, `warning`, and `error`.
- `npm run tauri build` creates Windows release artifacts under `src-tauri/target/release/`.
- The CI validation commands, run from `routelight/`, are:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
  - `cargo check --manifest-path src-tauri/Cargo.toml --locked`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml --locked`
  - `npm run build`

## Coding Style & Naming Conventions

Use two-space indentation in JavaScript/CSS and standard `rustfmt` formatting for Rust. Keep frontend code as plain ES modules and interact with backend commands through `window.__TAURI__`. Use `camelCase` for JavaScript variables/functions, `snake_case` for Rust functions/modules, and descriptive Tauri command names such as `refresh_status`. Keep user-visible Chinese/English UI labels consistent with the existing bilingual style.

## Testing Guidelines

Focused Rust unit tests are already committed. For Rust behavior changes, add or update focused tests near the relevant module and run relevant existing tests with `cargo test --manifest-path src-tauri/Cargo.toml --locked`. CI is the authoritative repository gate for Rust format, check, Clippy, tests, and the Tauri build using the commands above. Frontend or tray behavior that is inherently visual or native should also receive manual verification when relevant, using `npm run tauri dev` and the mock status environment variable. Network-probe changes should document which live endpoints were exercised and whether mock mode was used.

## Commit & Pull Request Guidelines

Use concise Conventional Commit-style messages, for example `feat: add IPv6 warning probe` or `fix: avoid duplicate refresh notification`. Pull requests should include a short summary, verification steps, linked issue or task context, and screenshots/GIFs for panel or tray UI changes. Call out any new network endpoint, permission, or Windows-specific behavior.

## Security & Configuration Tips

RouteLight must remain non-intrusive: do not add proxy mutation, route-table changes, packet capture, credential access, clipboard reads, or shell-command execution. Keep diagnostics in memory unless a user explicitly copies them. Never commit secrets, local `.env` files, build outputs, or `src-tauri/target/`.
