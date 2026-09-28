# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| v0.4.x  | Yes       |
| v0.3.x  | No        |
| v0.2.x  | No        |
| v0.1.x  | No        |

## Reporting a Vulnerability

Please do not report undisclosed vulnerabilities in public Issues. Use the repository's Security tab and the “Report a vulnerability” flow to submit a private vulnerability report.

Include the affected version and a concise description, reproduction steps if available, and relevant logs or screenshots only when they do not contain secrets.

## Security Boundaries

RouteLight is designed as a read-only diagnostic tray utility. It should remain non-intrusive:

- It does not capture packets.
- It does not read clipboard contents.
- It writes diagnostic text to the clipboard only after an explicit user action: Copy Diagnostics / 复制诊断 or AI diagnostic context / AI 上下文.
- It does not upload diagnostic reports.
- It does not write diagnostic reports to disk.
- It does not modify Windows proxy settings.
- It does not modify routing tables.
- It does not switch VPN/proxy nodes.
- It does not read browser cookies or credentials.
- It does not create scheduled tasks.
- It does not register startup entries by default. Only when the user explicitly enables "随系统启动" does RouteLight use the official Tauri autostart plugin to write system autostart configuration, and the user can disable it at any time.

Changes that expand system permissions, add shell execution, read local files, read clipboard contents, or upload diagnostics require explicit security review.
