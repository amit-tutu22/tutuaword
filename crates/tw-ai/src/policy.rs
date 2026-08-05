use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct RateLimit {
    pub requests_per_minute: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Default)]
pub struct AiPolicy {
    pub allow_cloud: bool,
    pub allow_local: bool,
    pub blocked_providers: HashSet<String>,
    pub require_local_for_classification: Option<DataClassification>,
    pub max_tokens_per_request: u32,
    pub rate_limit: Option<RateLimit>,
}

impl AiPolicy {
    pub fn local_only() -> Self {
        Self {
            allow_cloud: false,
            allow_local: true,
            ..Default::default()
        }
    }

    pub fn is_provider_allowed(&self, provider_id: &str) -> bool {
        !self.blocked_providers.contains(provider_id)
    }
}
