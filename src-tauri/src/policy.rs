use crate::traits::PolicyEnforcer;

pub struct PolicyManager {
    allowed_domains: Vec<String>,
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyManager {
    pub fn new() -> Self {
        Self {
            allowed_domains: vec![
                "notion.so".to_string(),
                "www.notion.so".to_string(),
                "notion.com".to_string(),
                "www.notion.com".to_string(),
                "msgstore.www.notion.so".to_string(),
                "file.notion.so".to_string(),
            ],
        }
    }

    fn is_official_notion(&self, host: &str) -> bool {
        self.allowed_domains
            .iter()
            .any(|domain| host == domain || host.ends_with(&format!(".{}", domain)))
    }
}

impl PolicyEnforcer for PolicyManager {
    fn validate_url(&self, url: &str) -> bool {
        log::debug!("PolicyManager: Validating URL: {}", url);
        // Parse URL and check if host is in allowed_domains
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                if self.is_official_notion(host) {
                    log::debug!("PolicyManager: ALLOWED official Notion domain: {}", host);
                    return true;
                }

                // Allow Google and Apple login navigation during OAuth flow
                if (host == "accounts.google.com" || host.ends_with(".accounts.google.com"))
                    || (host == "appleid.apple.com" || host.ends_with(".appleid.apple.com"))
                {
                    log::debug!("PolicyManager: ALLOWED OAuth provider: {}", host);
                    return true;
                }

                log::warn!("PolicyManager: BLOCKED unauthorized endpoint: {}", host);
                return false;
            }
        }

        log::warn!(
            "PolicyManager: BLOCKED malformed or unsupported URL: {}",
            url
        );
        // Block other protocols/malformed URLs by default (Zero-Trust)
        false
    }

    fn telemetry_allowed(&self) -> bool {
        // Manifesto Part III: Anti-Telemetry by Default
        false
    }

    fn validate_external_link(&self, url: &str) -> bool {
        // For external links, we apply a broader security check.
        // For now, allow https only and block common malicious protocols.
        if let Ok(parsed_url) = url::Url::parse(url) {
            // Block known tracker and telemetry domains from jumping out to the browser
            if let Some(host) = parsed_url.host_str() {
                if host.contains("googletagmanager.com")
                    || host.contains("google-analytics.com")
                    || host.contains("amplitude.com")
                    || host.contains("mixpanel.com")
                    || host.contains("segment.com")
                {
                    log::warn!("PolicyManager: BLOCKED tracker/telemetry domain: {}", host);
                    return false;
                }
            }

            match parsed_url.scheme() {
                "https" => true,
                "mailto" => true,
                _ => {
                    log::warn!(
                        "PolicyManager: BLOCKED unsafe external link protocol: {}",
                        parsed_url.scheme()
                    );
                    false
                }
            }
        } else {
            false
        }
    }

    fn should_route_popup_to_system_browser(&self, url: &str) -> bool {
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                // If it's an official Notion domain or OAuth provider, keep it in the app
                if self.is_official_notion(host)
                    || (host == "accounts.google.com" || host.ends_with(".accounts.google.com"))
                    || (host == "appleid.apple.com" || host.ends_with(".appleid.apple.com"))
                    || (host == "apple.com" || host.ends_with(".apple.com"))
                {
                    return false;
                }
            }

            // Local tauri schemes stay in-app
            if parsed_url.scheme() == "tauri" {
                return false;
            }
        }

        // Everything else is an external link and should open in system browser
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PolicyEnforcer;

    #[test]
    fn test_should_route_popup_to_system_browser() {
        let policy = PolicyManager::new();
        // OAuth providers should NOT be routed (they stay in app to capture session)
        assert!(!policy
            .should_route_popup_to_system_browser("https://accounts.google.com/o/oauth2/v2/auth"));
        assert!(!policy.should_route_popup_to_system_browser("https://appleid.apple.com/auth"));

        // External links should be routed
        assert!(policy.should_route_popup_to_system_browser("https://github.com/login"));

        // Notion internal popups should NOT be routed
        assert!(!policy.should_route_popup_to_system_browser("https://www.notion.so/some-popup"));
        assert!(!policy.should_route_popup_to_system_browser("https://www.notion.com/some-popup"));

        // Local tauri schemes should NOT be routed
        assert!(!policy.should_route_popup_to_system_browser("tauri://localhost/index.html"));
    }

    #[test]
    fn test_validate_official_notion_urls() {
        let policy = PolicyManager::new();
        assert!(policy.validate_url("https://www.notion.so"));
        assert!(policy.validate_url("https://notion.com/some-page"));
        assert!(policy.validate_url("https://accounts.google.com/auth"));
        assert!(policy.validate_url("https://appleid.apple.com/auth"));
    }

    #[test]
    fn test_block_unauthorized_urls() {
        let policy = PolicyManager::new();
        assert!(!policy.validate_url("https://google.com"));

        // Exact substring attack tests - should be blocked
        assert!(!policy.validate_url("https://evilnotion.so"));
        assert!(!policy.validate_url("https://hacker.evilnotion.so"));
        assert!(!policy.validate_url("https://accounts.google.com.evil.com"));
        assert!(!policy.validate_url("https://malicious-site.com/notion.so"));

        assert!(!policy.validate_url("http://localhost:3000"));
    }

    #[test]
    fn test_telemetry_always_blocked() {
        let policy = PolicyManager::new();
        assert!(!policy.telemetry_allowed());
    }

    #[test]
    fn test_validate_external_links() {
        let policy = PolicyManager::new();
        assert!(
            policy.validate_external_link("https://github.com/diegoakanottheoperator/lotion-rs")
        );
        assert!(policy.validate_external_link("mailto:support@notion.so"));
        assert!(!policy.validate_external_link("http://unsecure-link.com"));
        assert!(!policy.validate_external_link("javascript:alert('XSS')"));
    }
}
