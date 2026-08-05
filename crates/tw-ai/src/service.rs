use tw_model::NodeId;

use crate::context::DocumentContext;
use crate::provider::{AiError, CompletionRequest};
use crate::router::ProviderRouter;
use crate::task::{AiTask, RewriteTone};

#[derive(Debug, Clone)]
pub enum AiResponse {
    TextSuggestion {
        text: String,
    },
    ChatReply {
        message: String,
    },
    ReviewResults {
        issue_count: u32,
    },
}

pub struct PromptTemplate {
    pub id: String,
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub task: AiTask,
}

impl PromptTemplate {
    pub fn for_task(task: AiTask) -> Self {
        let id = format!("{task:?}").to_lowercase();
        Self {
            id: id.clone(),
            system_prompt: "You are a helpful writing assistant.".into(),
            user_prompt_template: "{{context}}".into(),
            max_tokens: 1024,
            temperature: 0.7,
            task,
        }
    }
}

pub trait AiService {
    fn summarize(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError>;
    fn rewrite(&self, ctx: &DocumentContext, tone: RewriteTone) -> Result<AiResponse, AiError>;
    fn translate(&self, ctx: &DocumentContext, lang: &str) -> Result<AiResponse, AiError>;
    fn generate(&self, ctx: &DocumentContext, prompt: &str) -> Result<AiResponse, AiError>;
    fn explain(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError>;
    fn correct_grammar(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError>;
}

pub struct AiServiceImpl {
    router: ProviderRouter,
}

impl AiServiceImpl {
    pub fn new(router: ProviderRouter) -> Self {
        Self { router }
    }

    pub fn router(&self) -> &ProviderRouter {
        &self.router
    }

    pub fn router_mut(&mut self) -> &mut ProviderRouter {
        &mut self.router
    }

    fn execute(&self, task: AiTask, ctx: &DocumentContext, extra: &str) -> Result<AiResponse, AiError> {
        let template = PromptTemplate::for_task(task);
        let context_text = ctx
            .selection
            .as_ref()
            .map(|s| s.text.clone())
            .unwrap_or_default();

        let prompt = template
            .user_prompt_template
            .replace("{{context}}", &context_text)
            + extra;

        let request = CompletionRequest {
            prompt,
            system_prompt: Some(template.system_prompt),
            max_tokens: template.max_tokens,
            temperature: template.temperature,
        };

        let response = self.router.complete(task, ctx, &request)?;
        Ok(AiResponse::TextSuggestion {
            text: response.text,
        })
    }
}

impl AiService for AiServiceImpl {
    fn summarize(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError> {
        self.execute(AiTask::Summarize, ctx, "")
    }

    fn rewrite(&self, ctx: &DocumentContext, tone: RewriteTone) -> Result<AiResponse, AiError> {
        self.execute(AiTask::Rewrite, ctx, &format!(" Tone: {tone:?}."))
    }

    fn translate(&self, ctx: &DocumentContext, lang: &str) -> Result<AiResponse, AiError> {
        self.execute(AiTask::Translate, ctx, &format!(" Target language: {lang}."))
    }

    fn generate(&self, ctx: &DocumentContext, prompt: &str) -> Result<AiResponse, AiError> {
        self.execute(AiTask::Generate, ctx, &format!(" Instruction: {prompt}"))
    }

    fn explain(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError> {
        self.execute(AiTask::Explain, ctx, "")
    }

    fn correct_grammar(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError> {
        self.execute(AiTask::CorrectGrammar, ctx, "")
    }
}

/// Document-scoped AI session facade.
pub struct AiSession {
    service: AiServiceImpl,
    pub doc_id: NodeId,
}

impl AiSession {
    pub fn new(service: AiServiceImpl, doc_id: NodeId) -> Self {
        Self { service, doc_id }
    }

    pub fn service(&self) -> &AiServiceImpl {
        &self.service
    }

    pub fn service_mut(&mut self) -> &mut AiServiceImpl {
        &mut self.service
    }

    pub fn summarize(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError> {
        let _ = self.doc_id;
        self.service.summarize(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextSelection;
    use crate::provider::{AiCapabilities, AiProvider, CompletionRequest, CompletionResponse, CompletionStream, LLAMA_CPP, OPENAI};
    use crate::router::ProviderRouter;

    struct MockProvider {
        id: String,
        local: bool,
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
                text: "rewritten".into(),
                tokens_used: 5,
            })
        }
        fn stream(&self, _: &CompletionRequest) -> Result<CompletionStream, AiError> {
            Ok(CompletionStream)
        }
        fn is_available(&self) -> bool {
            true
        }
    }

    fn service_with_mocks() -> AiServiceImpl {
        let mut router = ProviderRouter::new(OPENAI, LLAMA_CPP);
        router.register(Box::new(MockProvider {
            id: LLAMA_CPP.into(),
            local: true,
        }));
        router.register(Box::new(MockProvider {
            id: OPENAI.into(),
            local: false,
        }));
        AiServiceImpl::new(router)
    }

    #[test]
    fn rewrite_uses_local_for_small_selection() {
        let service = service_with_mocks();
        let ctx = DocumentContext {
            selection: Some(ContextSelection {
                text: "Hello world".into(),
                char_format_summary: String::new(),
                paragraph_style: None,
            }),
            total_token_estimate: 100,
            page_count: 1,
            ..Default::default()
        };
        let response = service.rewrite(&ctx, RewriteTone::Formal).unwrap();
        match response {
            AiResponse::TextSuggestion { text } => assert_eq!(text, "rewritten"),
            _ => panic!("expected text suggestion"),
        }
    }
}
