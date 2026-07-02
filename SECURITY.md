# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| v0.1.x  | Yes       |

## Reporting a Vulnerability

Please do not disclose security vulnerabilities in public issues before the maintainer has had time to review them.

For now, please report security issues by opening a private contact channel with the maintainer before publishing details. Include a concise description, reproduction steps if available, affected version, and any relevant logs or screenshots that do not contain secrets.

## Security Boundaries

RouteLight is designed as a read-only diagnostic tray utility. It should remain non-intrusive:

- It does not capture packets.
- It does not read clipboard contents.
- It writes to the clipboard only when the user clicks Copy Diagnostics.
- It does not upload diagnostic reports.
- It does not write diagnostic reports to disk.
- It does not modify Windows proxy settings.
- It does not modify routing tables.
- It does not switch VPN/proxy nodes.
- It does not read browser cookies or credentials.
- It does not create scheduled tasks.
- It does not register startup entries by default. Only when the user explicitly enables "随系统启动" does RouteLight use the official Tauri autostart plugin to write system autostart configuration, and the user can disable it at any time.

Changes that expand system permissions, add shell execution, read local files, read clipboard contents, or upload diagnostics require explicit security review.
