#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AiTask {
    SpellCheck,
    Grammar,
    Rewrite,
    Translate,
    Summarize,
    Generate,
    Explain,
    CorrectGrammar,
    Chat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiRoutingMode {
    AlwaysLocal,
    AlwaysCloud,
    #[default]
    Automatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteTone {
    Neutral,
    Formal,
    Casual,
    Shorten,
    Expand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiPlatform {
    #[default]
    Desktop,
    MobileApple,
    MobileAndroid,
}

impl AiTask {
    /// Whether this task prefers a local provider in Automatic mode.
    pub fn prefers_local(&self, page_count: u32, token_estimate: u32) -> bool {
        match self {
            AiTask::SpellCheck => true,
            AiTask::Grammar | AiTask::CorrectGrammar | AiTask::Rewrite => true,
            AiTask::Translate | AiTask::Explain => token_estimate < 4_000,
            AiTask::Summarize => page_count <= 5 && token_estimate < 8_000,
            AiTask::Generate => token_estimate < 2_000,
            AiTask::Chat => token_estimate < 4_000,
        }
    }
}
