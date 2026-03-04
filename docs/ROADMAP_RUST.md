# Roadmap: Porting Lotion to Rust

This document outlines the strategic plan for porting Lotion from an Electron-based application to a native Rust application using **Tauri v2** for webview management and **Iced** for native UI chrome.

## 🎯 Objectives
- **Performance**: Significant reduction in memory usage and faster startup times.
- **Security**: Leveraging Rust's memory safety, LiteBox sandboxing, and Zero-Trust policies.
- **Maintainability**: Moving core logic to a type-safe language with SOLID architecture.
- **Native Experience**: Iced-based UI chrome with system-level integration.

---

## 🏗️ Architecture (Tauri v2 + Iced Hybrid)

- **Backend (Rust/Tauri)**: Core logic, webview management, tab orchestration, security policies, theming engine.
- **Frontend (Iced)**: Native tab bar, navigation controls, window controls — no web technologies for chrome.
- **Webview (WRY)**: Rendering Notion.so using Tauri's multi-webview capabilities as child views.
- **Bridge**: `tokio::sync::mpsc` channels for cross-thread communication between Iced (main thread) and Tauri (background thread).

---

## 📁 Project File Index

| File | Purpose | Status |
| :--- | :--- | :--- |
| `src-tauri/src/main.rs` | Entry point: initializes modules, spawns Tauri, runs Iced | ✅ Done |
| `src-tauri/src/lib.rs` | Module declarations | ✅ Done |
| `src-tauri/src/traits.rs` | Trait abstractions: `SecuritySandbox`, `TabOrchestrator`, `WindowProvider`, `PolicyEnforcer`, `ThemingEngine` | ✅ Done |
| `src-tauri/src/state.rs` | `AppState`, `WindowState`, `Bounds` — serializable app state | ✅ Done |
| `src-tauri/src/security.rs` | `SecurityModule` — wraps `LiteBox` sandbox | ✅ Done |
| `src-tauri/src/policy.rs` | `PolicyManager` — Zero-Trust URL whitelisting & anti-telemetry | ✅ Done |
| `src-tauri/src/window_controller.rs` | `WindowController` — frameless window, event listeners | ✅ Done |
| `src-tauri/src/tab_controller.rs` | `TabController` — webview lifecycle, navigation interception, theme injection | ✅ Done |
| `src-tauri/src/tab_manager.rs` | `TabManager` — multi-tab orchestration via `TabOrchestrator` trait | ✅ Done |
| `src-tauri/src/menu.rs` | Native menu (Lotion, Navigation, Edit, View) via `MenuBuilder` | ✅ Done |
| `src-tauri/src/ui/mod.rs` | `LotionApp` (Iced `Application`), message handling, Tauri subscription | ✅ Done |
| `src-tauri/src/ui/tab_bar.rs` | Iced tab bar widgets: nav controls, dynamic tabs, window controls | ✅ Done |
| `src-tauri/src/ui/theme.rs` | Iced color palette constants (pixel-perfect legacy match) | ✅ Done |
| `src-tauri/src/ui/theming.rs` | `ThemeManager` — CSS injection for Dracula/Nord themes | ✅ Done |
| `src-tauri/src/litebox/` | LiteBox sandboxing library (7 files: core, platform, host, fd, sync, blb1) | ✅ Done |
| `docs/MANIFESTO.md` | Zero-Trust Manifesto | ✅ Done |
| `docs/DECISIONS.md` | Architectural Decision Records (ADR 1-7) | ✅ Done |
| `docs/ROADMAP_RUST.md` | This file | ✅ Done |

---

## 📋 Legacy Migration Checklist

### Main Process (Backend)
- [x] **Project Scaffolding**: `cargo init`, `src-tauri/` directory structure.
- [x] **Sandboxing**: LiteBox core ported to `src-tauri/src/litebox/`.
- [x] **Window Controller**: Frameless window creation, event listeners (`WindowController`).
- [x] **Tab Manager**: Multi-tab orchestration (`TabManager` + `TabController`).
- [x] **Tab Controller**: Webview lifecycle, Zero-Trust navigation interception, CSS theme injection.
- [x] **Native Menus**: Lotion, Navigation, Edit, View menus via `MenuBuilder`.
- [x] **State Management**: `AppState` with `WindowState` and `Bounds` (replaces Redux slices).
- [x] **Zero-Trust Policy**: `PolicyManager` with Notion domain whitelisting and anti-telemetry.
- [x] **Secure External Links**: Validated link opening via `opener` crate.
- [x] **App Lifecycle**: Multi-instance handling, graceful quit, state persistence.

### Frontend (Iced Native UI)
- [x] **Tab Bar UI**: Native Iced tab bar with navigation, tabs, window controls.
- [x] **Dynamic Tabs**: Tab rendering driven by `Vec<TabInfo>` state.
- [x] **Iced Application**: Transitioned from `Sandbox` to `Application` with `Command` and `Subscription`.
- [x] **Tauri Bridge**: `mpsc` channel subscription for cross-thread Tauri events.
- [x] **WindowController Integration**: `WindowController` initialized on `TauriReady` message.
- [x] **Theme Constants**: Pixel-perfect color palette matching legacy Electron app.

### Theming & Customization
- [x] **Dracula Theme**: CSS variable injection via `webview.eval()`.
- [x] **Nord Theme**: CSS variable injection via `webview.eval()`.
- [x] **Custom User CSS**: Load and inject user-defined CSS from disk.
- [x] **Theme Persistence**: Save/load active theme from config.
- [x] **Live Theme Switching**: Change theme without restarting.

### Resources & Config
- [x] **Assets**: Migrate icons and branding from `assets/`.
- [x] **Config**: Implement config file (TOML/JSON) for user preferences.
- [x] **I18n**: Migrate localization strings.

---

## 🗓️ Phase 1: Foundation & Core Logic — ✅ COMPLETE
- [x] Project initialization with `cargo init`.
- [x] LiteBox security sandbox integration.
- [x] `SecuritySandbox` trait and `SecurityModule` implementation.
- [x] Trait-based SOLID architecture (`traits.rs`).

## 🗓️ Phase 2: Multi-Tab Support — ✅ COMPLETE
- [x] `TabController`: Individual webview management.
- [x] `TabManager`: Multi-tab orchestration via `TabOrchestrator` trait.
- [x] `WindowController`: Frameless window with event listeners.

## 🗓️ Phase 3: Features & Integration — ✅ COMPLETE
- [x] **Theming Engine**: CSS injection for Dracula/Nord themes.
- [x] **Native Menus**: Full menu system via Tauri v2 `MenuBuilder`.
- [x] **External Link Handling**: Secure URL validation and system browser opening.

## 🗓️ Phase 4: Tauri-Iced Integration — ✅ COMPLETE
- [x] Iced `Application` with stateful `LotionApp`.
- [x] `tokio::sync::mpsc` bridge between Tauri and Iced threads.
- [x] `iced::subscription::channel` for streaming Tauri events into UI.
- [x] `WindowController` initialized from `TauriReady` handler.
- [x] Dynamic tab rendering from `Vec<TabInfo>` state.

## 🗓️ Phase 5: Iced Frontend Implementation — ✅ COMPLETE
- [x] Native tab bar widgets in `tab_bar.rs`.
- [x] Color palette in `theme.rs` (pixel-perfect legacy match).
- [x] Navigation controls, window controls, app logo.
- [x] Frameless/transparent window configuration.

## 🗓️ Phase 6: Zero-Trust Manifesto Enforcement — ✅ COMPLETE
- [x] `PolicyManager` with Notion domain whitelist.
- [x] Anti-telemetry enforcement (`telemetry_allowed() -> false`).
- [x] Navigation interception in `TabController` with fail-closed behavior.
- [x] External link protocol validation (HTTPS/mailto only).
- [x] `PolicyEnforcer` trait and integration.

## 🗓️ Phase 7: Persistence & Polish — ✅ COMPLETE
- [x] **Config Persistence**: `LotionConfig` with TOML at `~/.config/lotion/config.toml`.
- [x] **State Persistence**: `AppState` save/restore via JSON at `~/.config/lotion/state.json`.
- [x] **Custom User CSS**: File-based loading in `ThemeManager::get_custom_css()`.
- [x] **Live Theme Switching**: `Message::ThemeChanged` + `set_active_theme()`/`get_active_theme()`.
- [x] **Tab Title Sync**: `MutationObserver` injected via `webview.eval()` in `TabController`.

## 🗓️ Phase 8: Distribution & CI/CD — ✅ COMPLETE
- [x] **CI Pipeline**: GitHub Actions with `cargo check`, `clippy`, `fmt`, `test`, `cargo-audit`.
- [x] **Build Pipeline**: Multi-platform Tauri build matrix (Linux x86_64, macOS x64/ARM64).
- [x] **Release Pipeline**: Automated GitHub Releases via `softprops/action-gh-release`.
- [x] **Bundler Config**: Tauri targets: DEB, RPM, AppImage.

---

## 🛠️ Key Technology Mappings

| Feature | Electron Implementation | Rust/Iced Equivalent |
| :--- | :--- | :--- |
| **Runtime** | Node.js / Chromium | Rust / WRY (System Webview) |
| **Windowing** | BrowserWindow | Iced Window + Tauri (background) |
| **Tabs** | WebContentsView | tauri::Webview (child views) |
| **UI Framework** | React / TypeScript | Iced (Native Rust UI) |
| **State** | Redux (Main Process) | `AppState` struct + `mpsc` channels |
| **Storage** | better-sqlite3 / electron-store | rusqlite / tauri-plugin-store |
| **Spell Check** | Electron Spellchecker | hunspell / zbus |
| **Security** | Electron sandbox | LiteBox + PolicyManager |
| **Theming** | insertCSS() IPC | webview.eval() from Rust backend |
| **Menus** | Electron Menu | Tauri v2 MenuBuilder |
| **Packaging** | Electron Forge | tauri-bundler |

---

## ⚠️ Challenges & Risks
1. **Multi-webview Complexity**: Tauri v2 required for stable multi-webview support.
2. **Notion Compatibility**: CSS/JS injection must adapt as Notion updates.
3. **System Dependencies**: Managing `webkit2gtk` across Linux distributions.
4. **Iced-Tauri Threading**: Two event loops require careful channel-based synchronization.
5. **State Consistency**: Ensuring `AppState` stays in sync across both threads.
