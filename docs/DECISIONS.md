# Architectural Decisions Log

This document records the key architectural decisions made during the porting of Lotion to Rust.

## ADR 1: Transition to Iced for Native UI
- **Date**: 2026-03-03
- **Decision**: Replace the planned React/Webview frontend for the "Application Chrome" (tabs, navigation) with the **Iced** library.
- **Rationale**: 
    - **Performance**: Eliminates the overhead of an extra renderer process for the UI.
    - **Security**: Reduces the attack surface by minimizing webview usage to only the Notion content.
    - **Native Experience**: Provides a truly native Rust UI that is type-safe and performant.

## ADR 2: Zero-Trust Manifesto Enforcement
- **Date**: 2026-03-03
- **Decision**: Implement a strict `PolicyManager` that blocks all remote interactions by default.
- **Exception**: Official `notion.so` endpoints are whitelisted to allow core functionality.
- **Rationale**: Aligns with the Zero-Trust Engineering Manifesto (docs/MANIFESTO.md) to minimize blast radius and exfiltration risks.

## ADR 3: SOLID Dependency Inversion
- **Date**: 2026-03-03
- **Decision**: Use trait-based dependency inversion (`SecuritySandbox`, `TabOrchestrator`, `PolicyEnforcer`) for all core components.
- **Rationale**: Ensures the backend is decoupled, testable, and allows for easy swapping of implementations (e.g., Mocks for testing).

## ADR 4: Standard Tauri v2 Project Structure
- **Date**: 2026-03-03
- **Decision**: Move the Rust backend and configuration into the standard `src-tauri` directory.
- **Rationale**: Follows community standards and prepares the project for better integration with Tauri's build tools and future frontend scaffolding.

## ADR 6: Native Rust Theming Engine
- **Date**: 2026-03-03
- **Decision**: Port the legacy CSS theme injection logic to a native Rust `ThemingEngine`. Themes are injected into Notion webviews using `webview.eval()` with a self-invoking JavaScript wrapper.
- **Rationale**: 
    - **Portability**: Allows re-using legacy theme colors while moving the orchestration to Rust.
    - **Security**: Centralizes CSS injection in the secured backend, validated by the `PolicyEnforcer`.
    - **Performance**: Native Rust theme management is more efficient than the legacy IPC-based injection.

## ADR 7: Native Menu with MenuBuilder
- **Date**: 2026-03-03
- **Decision**: Implement the application menu using Tauri v2's `MenuBuilder` and `SubmenuBuilder` in `src/menu.rs`.
- **Rationale**: 
    - **Native Look**: Ensures the app feels at home on Linux (and other platforms) by using OS-standard menu components.
    - **Performance**: Event handling occurs directly in the Rust main loop, avoiding IPC round-trips for menu actions.
    - **Type Safety**: Leverages Rust's type system for menu structure and event dispatching.

## ADR 8: Tauri-Iced Hybrid Threading Model
- **Date**: 2026-03-03
- **Decision**: Run Tauri in a background `std::thread` and Iced as the primary UI loop on the main thread. Communication uses `tokio::sync::mpsc` channels bridged into Iced via `iced::subscription::channel`.
- **Rationale**: 
    - **Decoupling**: Allows Iced to own the main thread (required by most windowing systems) while Tauri manages webviews independently.
    - **Performance**: Eliminates blocking between the UI rendering and webview I/O operations.
    - **Extensibility**: The `mpsc` channel pattern makes it trivial to add new message types for future features.

## ADR 9: TOML Config + JSON State Persistence
- **Date**: 2026-03-03
- **Decision**: User preferences stored as TOML (`config.toml`) and runtime state as JSON (`state.json`), both in `~/.config/lotion/`.
- **Rationale**: 
    - **Human-Readable**: TOML is ideal for user-editable configuration; JSON is ideal for serialized AppState.
    - **Separation of Concerns**: Config (theme, CSS path) is user intent; State (window bounds, tabs) is runtime data.
    - **Resilience**: Load failures fall back to defaults (fail-open for UX, not security).

## ADR 10: Rust-Native CI/CD Pipeline
- **Date**: 2026-03-03
- **Decision**: Replace all legacy Node.js/Electron CI workflows with Rust-native tooling: `cargo check`, `clippy`, `fmt`, `test`, `cargo-audit` for testing, and `cargo tauri build` for releases.
- **Rationale**: 
    - **Consistency**: The entire project is now Rust; CI should match.
    - **Security**: `cargo-audit` catches known CVEs in dependencies.
    - **Multi-Platform**: Tauri's bundler produces DEB, RPM, and AppImage for Linux, and DMG for macOS.
