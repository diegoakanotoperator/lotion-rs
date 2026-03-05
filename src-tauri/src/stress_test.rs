#[cfg(test)]
mod stress_tests {
    use crate::policy::PolicyManager;
    use crate::state::AppState;
    use crate::traits::PolicyEnforcer;

    #[test]
    fn stress_test_policy_manager() {
        let policy = PolicyManager::new();

        // 1,000 rapid URL validations
        for i in 0..1000 {
            let url = format!("https://www.notion.so/page-{}", i);
            assert!(policy.validate_url(&url));

            let evil_url = format!("https://evil-site-{}.com/notion.so", i);
            assert!(!policy.validate_url(&evil_url));

            let tracker = "https://www.googletagmanager.com/gtm.js";
            assert!(!policy.validate_external_link(tracker));
        }
    }

    #[test]
    fn stress_test_app_state_serialization() {
        let mut app_state = AppState::new();

        // Simulate 100 windows with 10 tabs each
        for w in 0..100 {
            let window_id = format!("window-{}", w);
            let mut tab_ids = Vec::new();
            for t in 0..10 {
                let tab_id = format!("{}-tab-{}", window_id, t);
                app_state.tabs.insert(
                    tab_id.clone(),
                    crate::state::TabState {
                        id: tab_id.clone(),
                        title: format!("Tab {}", t),
                        url: "https://www.notion.so".to_string(),
                        is_active: t == 0,
                        is_pinned: false,
                    },
                );
                tab_ids.push(tab_id);
            }

            app_state.windows.insert(
                window_id.clone(),
                crate::state::WindowState {
                    id: window_id,
                    bounds: crate::state::Bounds {
                        x: Some(0.0),
                        y: Some(0.0),
                        width: 800.0,
                        height: 600.0,
                    },
                    is_focused: false,
                    is_maximized: false,
                    is_minimized: false,
                    is_full_screen: false,
                    tab_ids,
                    active_tab_id: None,
                },
            );
        }

        // Test serialization performance/stability
        let json = serde_json::to_string(&app_state).expect("Failed to serialize large state");
        let decoded: AppState =
            serde_json::from_str(&json).expect("Failed to deserialize large state");

        assert_eq!(decoded.windows.len(), 100);
        assert_eq!(decoded.tabs.len(), 1000);
    }

    #[test]
    fn stress_test_policy_malformed_input() {
        let policy = PolicyManager::new();
        let cases = vec![
            "https://notion.so/󿿿",
            "notion.so",
            "javascript:alert(1)",
            "data:text/html,base64...",
            "file:///etc/passwd",
            "",
            "https://www.notion.so@evil.com",
            "https://accounts.google.com.evil.com",
        ];

        for case in cases {
            // Should never panic, and should block most of these
            let _ = policy.validate_url(case);
        }
    }
}
