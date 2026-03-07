# Security Policy

`lotion-rs` is built with a "Zero-Trust" security philosophy. We take the security of your data and your system seriously. This document outlines our security features, supported versions, and how to report vulnerabilities.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| < 0.2.0 | :x:                |

We only provide security patches for the latest minor release of the current major version.

## Core Security Features (LiteBox Architecture)

The application implements several layers of defense-in-depth to protect users from malicious content within the Notion webview:

### 1. OS-Level Sandboxing (LiteBox V3)
- **Linux**: Full **Namespace Isolation** using `unshare(2)`. The application runs in private `Mount`, `UTS`, `IPC`, `PID`, and `Network` namespaces.
- **Windows**: **Job Object** restrictions. Limits UI interactivity and restricts child process spawning to prevent unauthorized execution.

### 2. Zero-Trust WebView Policies
- **HTTPS Enforcement**: All navigations are strictly limited to `https://` schemes.
- **Strict Domain Matching**: Protection against suffix attacks (e.g., `evilnotion.so`) using exact and subdomain-prefixed matching.
- **Secure Popups**: Every window (popup or tab) created by the application inherits strict security listeners for navigation and new-window events.

### 3. Supply Chain & Update Security
- **Signed Releases**: All release packages are cryptographically signed. The application verifies these signatures before applying any updates.
- **CI/CD Hardening**: GitHub Actions workflows use the principle of least privilege (`contents: read`).
- **Input Sanitization**: Strict validation of all external inputs, including locales and shell URLs, to prevent path traversal and protocol smuggling.

## Reporting a Vulnerability

**DO NOT open a public GitHub issue for security vulnerabilities.**

If you discover a security vulnerability, please report it privately. You can contact the maintainer directly through the following channels:

- **Email**: [diegoakanotoperator@users.noreply.github.com](mailto:diegoakanotoperator@users.noreply.github.com)
- **GitHub Private Vulnerability Reporting**: Use the "Report a vulnerability" button on the [repository's Security tab](https://github.com/diegoakanotoperator/lotion-rs/security/advisories/new) (preferred).

Please include as much information as possible, including:
- A description of the vulnerability.
- A proof-of-concept (PoC) or steps to reproduce.
- The impact of the vulnerability.

We aim to acknowledge receipt of your report within 48 hours and provide a timeline for remediation.

## Vulnerability Tracking

For a detailed history of identified and remediated vulnerabilities, please refer to [docs/security_issues.md](docs/security_issues.md).
