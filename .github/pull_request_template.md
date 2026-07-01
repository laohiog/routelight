## Summary

-

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `npm run tauri build`

## Security Boundary

- [ ] This PR does not add shell execution, filesystem access, clipboard reads, diagnostic uploads, proxy mutation, route changes, packet capture, startup registration, or scheduled tasks.
- [ ] New network endpoints, Tauri permissions, or Windows-specific behavior are described above, if any.

## Screenshots / Notes

Add screenshots or notes for UI, tray, notification, or installer changes.
