# RouteLight

RouteLight is a lightweight Windows Tauri tray utility for monitoring network route status and AI service reachability.

The application source lives in [`routelight/`](routelight/). Start with the app README:

- [RouteLight app README](routelight/README.md)
- [v0.2.0 release notes](routelight/RELEASE_NOTES_v0.2.0.md)
- [v0.1.0-rc1 release notes](routelight/RELEASE_NOTES_v0.1.0-rc1.md)
- [Technical blueprint](docs/technical-blueprint.md)

## Quick Start

```powershell
cd routelight
npm install
npm run tauri dev
```

## Build

```powershell
cd routelight
npm run tauri build
```

The default release artifact is the NSIS installer under `routelight/src-tauri/target/release/bundle/nsis/`.

## Security Boundary

RouteLight does not register startup entries by default. Only when the user explicitly enables "随系统启动" does RouteLight use the official Tauri autostart plugin to write system autostart configuration, and the user can disable it at any time.

## License

MIT License. See [LICENSE](LICENSE).
