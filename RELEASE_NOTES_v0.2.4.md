# Release Notes v0.2.4 - Security Hardening Update

## Summary
This release focuses on hardening the application's security posture by addressing several vulnerabilities identified during the security audit. We have implemented strict Zero-Trust policies for URL navigation, window creation, and locale management.

## Security Improvements
- **Full Namespace Isolation (Linux)**: The main process now enforces strict OS-level isolation using Mount, UTS, IPC, PID, and Network namespaces (LiteBox v2).
- **Secure Automated Updates**: Integrated the Tauri v2 updater with cryptographic signature verification (Minisign). Updates are now delivered via a secure GitHub-based channel.
- **Improved Security Defaults**: Updated application identifier and ensured all cryptographic keys use modern standards (non-Base64 raw keys where applicable).
- **Locale Sanitization**: Implemented strict input validation for the i18n manager to prevent path traversal attacks.
- **GitHub Actions Hardening**: Updated CI/CD workflows with explicit least-privilege permissions (`contents: read`).
- **Sandbox Status**: Verified WebKit sandbox enforcement and updated "LiteBox" documentation to accurately reflect its current experimental defense-in-depth status.

## Changes
- [SECURITY] Implement full OS-level namespace isolation (Mount, UTS, IPC, PID, NET) in `litebox/linux.rs`.
- [SECURITY] Harden `PolicyManager` domain matching (Strict suffix/subdomain check).
- [SECURITY] Centralize `WebviewBuilder` creation in `TabController` for consistent security policy application.
- [SECURITY] Add `validate_locale` helper to `i18n.rs`.
- [SECURITY] Update GitHub Actions workflows with explicit `permissions` and Tauri Updater support.
- [DOCS] Comprehensive update to `docs/security_issues.md` tracking all remediations.
- [BUMP] Version 0.2.4 with new secure identifier `io.lotion-rs.secure.v2`.
