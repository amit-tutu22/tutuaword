use std::collections::HashMap;

use crate::context::DocumentContext;
use crate::module::AiModuleRegistry;
use crate::policy::AiPolicy;
use crate::provider::{
    AiError, AiProvider, APPLE_FM, CLAUDE, CUSTOM, GEMINI, LLAMA_CPP, LITERT, ONNX, OPENAI,
    OPENROUTER, OLLAMA, RULES,
};
use crate::task::{AiPlatform, AiRoutingMode, AiTask};

pub struct HybridRouter {
    providers: HashMap<String, Box<dyn AiProvider>>,
    default_cloud: String,
    default_local: String,
    platform: AiPlatform,
    routing_mode: AiRoutingMode,
    policy: AiPolicy,
    modules: AiModuleRegistry,
}

impl HybridRouter {
    pub fn new(
        default_cloud: impl Into<String>,
        default_local: impl Into<String>,
        platform: AiPlatform,
    ) -> Self {
        Self {
            providers: HashMap::new(),
            default_cloud: default_cloud.into(),
            default_local: default_local.into(),
            platform,
            routing_mode: AiRoutingMode::Automatic,
            policy: AiPolicy {
                allow_cloud: true,
                allow_local: true,
                ..Default::default()
            },
            modules: AiModuleRegistry::with_defaults(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        self.providers.insert(provider.id().to_string(), provider);
    }

    pub fn with_routing_mode(mut self, mode: AiRoutingMode) -> Self {
        self.routing_mode = mode;
        self
    }

    pub fn with_policy(mut self, policy: AiPolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn routing_mode(&self) -> AiRoutingMode {
        self.routing_mode
    }

    pub fn set_routing_mode(&mut self, mode: AiRoutingMode) {
        self.routing_mode = mode;
    }

    pub fn policy(&self) -> &AiPolicy {
        &self.policy
    }

    pub fn modules(&self) -> &AiModuleRegistry {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> &mut AiModuleRegistry {
        &mut self.modules
    }

    /// Select a provider ID for the given task and document context.
    pub fn route(&self, task: AiTask, ctx: &DocumentContext) -> Result<String, AiError> {
        if task == AiTask::SpellCheck {
            return Ok(RULES.to_string());
        }

        if let Some(classification) = ctx.classification {
            if self.policy.require_local_for_classification == Some(classification) {
                return self.resolve_local(task);
            }
        }

        match self.routing_mode {
            AiRoutingMode::AlwaysLocal => self.resolve_local(task),
            AiRoutingMode::AlwaysCloud => self.resolve_cloud(),
            AiRoutingMode::Automatic => {
                if !self.policy.allow_cloud {
                    return self.resolve_local(task);
                }
                if !self.policy.allow_local {
                    return self.resolve_cloud();
                }
                if task.prefers_local(ctx.page_count, ctx.total_token_estimate) {
                    self.resolve_local(task).or_else(|_| self.resolve_cloud())
                } else {
                    self.resolve_cloud().or_else(|_| self.resolve_local(task))
                }
            }
        }
    }

    fn resolve_local(&self, task: AiTask) -> Result<String, AiError> {
        if !self.policy.allow_local {
            return Err(AiError::PolicyViolation(
                "local AI disabled by policy".into(),
            ));
        }

        let platform_local = match self.platform {
            AiPlatform::Desktop => LLAMA_CPP,
            AiPlatform::MobileApple => APPLE_FM,
            AiPlatform::MobileAndroid => ONNX,
        };

        for candidate in [platform_local, &self.default_local, LLAMA_CPP, ONNX, LITERT, OLLAMA] {
            if self.is_available(candidate) {
                let _ = self.modules.smallest_for_task(task);
                return Ok(candidate.to_string());
            }
        }

        Err(AiError::ProviderUnavailable(
            "no local provider available".into(),
        ))
    }

    fn resolve_cloud(&self) -> Result<String, AiError> {
        if !self.policy.allow_cloud {
            return Err(AiError::PolicyViolation(
                "cloud AI disabled by policy".into(),
            ));
        }

        for candidate in [
            &self.default_cloud,
            OPENAI,
            GEMINI,
            CLAUDE,
            OPENROUTER,
            CUSTOM,
        ] {
            if self.is_available(candidate) {
                return Ok(candidate.to_string());
            }
        }

        Err(AiError::ProviderUnavailable(
            "no cloud provider available".into(),
        ))
    }

    fn is_available(&self, provider_id: &str) -> bool {
        if !self.policy.is_provider_allowed(provider_id) {
            return false;
        }
        self.providers
            .get(provider_id)
            .map(|p| p.is_available())
            .unwrap_or(provider_id == RULES)
    }

    pub fn get_provider(&self, provider_id: &str) -> Option<&dyn AiProvider> {
        self.providers
            .get(provider_id)
            .map(|p| p.as_ref() as &dyn AiProvider)
    }
}

/// Legacy router kept for backward compatibility; delegates to HybridRouter internally.
pub struct ProviderRouter {
    hybrid: HybridRouter,
}

impl ProviderRouter {
    pub fn new(default_cloud: impl Into<String>, default_local: impl Into<String>) -> Self {
        Self {
            hybrid: HybridRouter::new(default_cloud, default_local, AiPlatform::Desktop),
        }
    }

    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        self.hybrid.register(provider);
    }

    pub fn with_policy(mut self, policy: AiPolicy) -> Self {
        self.hybrid = self.hybrid.with_policy(policy);
        self
    }

    pub fn policy(&self) -> &AiPolicy {
        self.hybrid.policy()
    }

    pub fn hybrid(&self) -> &HybridRouter {
        &self.hybrid
    }

    pub fn hybrid_mut(&mut self) -> &mut HybridRouter {
        &mut self.hybrid
    }

    pub fn complete(
        &self,
        task: AiTask,
        ctx: &DocumentContext,
        request: &crate::provider::CompletionRequest,
    ) -> Result<crate::provider::CompletionResponse, AiError> {
        let provider_id = self.hybrid.route(task, ctx)?;
        if provider_id == RULES {
            return Err(AiError::NotImplemented);
        }
        let provider = self
            .hybrid
            .get_provider(&provider_id)
            .ok_or_else(|| AiError::ProviderUnavailable(provider_id.clone()))?;
        provider.complete(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::DataClassification;
    use crate::provider::{AiCapabilities, AiProvider, CompletionRequest, CompletionResponse, CompletionStream};

    struct MockProvider {
        id: String,
        local: bool,
        available: bool,
    }

    impl AiProvider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.id
        }

        fn capabilities(&self) -> AiCapabilities {
            AiCapabilities {
                local: self.local,
                ..Default::default()
            }
        }

        fn complete(&self, _: &CompletionRequest) -> Result<CompletionResponse, AiError> {
            Ok(CompletionResponse {
                text: "ok".into(),
                tokens_used: 1,
            })
        }

        fn stream(&self, _: &CompletionRequest) -> Result<CompletionStream, AiError> {
            Ok(CompletionStream)
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    fn test_router() -> HybridRouter {
        let mut router = HybridRouter::new(OPENAI, LLAMA_CPP, AiPlatform::Desktop);
        router.register(Box::new(MockProvider {
            id: LLAMA_CPP.into(),
            local: true,
            available: true,
        }));
        router.register(Box::new(MockProvider {
            id: OPENAI.into(),
            local: false,
            available: true,
        }));
        router.register(Box::new(MockProvider {
            id: GEMINI.into(),
            local: false,
            available: true,
        }));
        router
    }

    fn small_ctx() -> DocumentContext {
        DocumentContext {
            total_token_estimate: 500,
            page_count: 1,
            ..Default::default()
        }
    }

    fn large_ctx() -> DocumentContext {
        DocumentContext {
            total_token_estimate: 120_000,
            page_count: 200,
            ..Default::default()
        }
    }

    #[test]
    fn grammar_routes_to_local_in_automatic_mode() {
        let router = test_router();
        let id = router.route(AiTask::Grammar, &small_ctx()).unwrap();
        assert_eq!(id, LLAMA_CPP);
    }

    #[test]
    fn long_summarize_routes_to_cloud_in_automatic_mode() {
        let router = test_router();
        let id = router.route(AiTask::Summarize, &large_ctx()).unwrap();
        assert_eq!(id, OPENAI);
    }

    #[test]
    fn always_local_blocks_cloud() {
        let router = test_router().with_routing_mode(AiRoutingMode::AlwaysLocal);
        let id = router.route(AiTask::Summarize, &large_ctx()).unwrap();
        assert_eq!(id, LLAMA_CPP);
    }

    #[test]
    fn policy_blocks_cloud() {
        let router = test_router().with_policy(AiPolicy::local_only());
        let id = router.route(AiTask::Summarize, &large_ctx()).unwrap();
        assert_eq!(id, LLAMA_CPP);
    }

    #[test]
    fn spell_check_uses_rules_engine() {
        let router = test_router();
        let id = router.route(AiTask::SpellCheck, &small_ctx()).unwrap();
        assert_eq!(id, RULES);
    }

    #[test]
    fn confidential_classification_forces_local() {
        let mut router = test_router();
        router.policy.require_local_for_classification = Some(DataClassification::Confidential);
        let mut ctx = large_ctx();
        ctx.classification = Some(DataClassification::Confidential);
        let id = router.route(AiTask::Summarize, &ctx).unwrap();
        assert_eq!(id, LLAMA_CPP);
    }
}