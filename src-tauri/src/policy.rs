use crate::traits::PolicyEnforcer;

pub struct PolicyManager {
    allowed_domains: Vec<String>,
}

impl PolicyManager {
    pub fn new() -> Self {
        Self {
            allowed_domains: vec![
                "notion.so".to_string(),
                "www.notion.so".to_string(),
                "msgstore.www.notion.so".to_string(),
                "file.notion.so".to_string(),
            ],
        }
    }

    fn is_official_notion(&self, host: &str) -> bool {
        self.allowed_domains.iter().any(|domain| host.ends_with(domain))
    }
}

impl PolicyEnforcer for PolicyManager {
    fn validate_url(&self, url: &str) -> bool {
        // Parse URL and check if host is in allowed_domains
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                if self.is_official_notion(host) {
                    return true;
                }
                
                log::warn!("PolicyManager: BLOCKED remote interaction with non-official endpoint: {}", host);
                return false;
            }
        }
        
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
            match parsed_url.scheme() {
                "https" => true,
                "mailto" => true,
                _ => {
                    log::warn!("PolicyManager: BLOCKED unsafe external link protocol: {}", parsed_url.scheme());
                    false
                }
            }
        } else {
            false
        }
    }
}
