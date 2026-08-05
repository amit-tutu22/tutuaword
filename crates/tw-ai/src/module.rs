use std::collections::HashMap;
use std::path::PathBuf;

use crate::task::AiTask;

#[derive(Debug, Clone)]
pub struct AiModule {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub tasks: Vec<AiTask>,
    pub model_path: Option<PathBuf>,
    pub installed: bool,
}

impl AiModule {
    pub fn grammar() -> Self {
        Self {
            id: "grammar".into(),
            name: "Grammar Module".into(),
            size_bytes: 200 * 1024 * 1024,
            tasks: vec![AiTask::CorrectGrammar, AiTask::Grammar],
            model_path: None,
            installed: false,
        }
    }

    pub fn translation() -> Self {
        Self {
            id: "translation".into(),
            name: "Translation Module".into(),
            size_bytes: 300 * 1024 * 1024,
            tasks: vec![AiTask::Translate],
            model_path: None,
            installed: false,
        }
    }

    pub fn writing() -> Self {
        Self {
            id: "writing".into(),
            name: "Writing Module".into(),
            size_bytes: 700 * 1024 * 1024,
            tasks: vec![AiTask::Rewrite, AiTask::Explain],
            model_path: None,
            installed: false,
        }
    }

    pub fn reasoning() -> Self {
        Self {
            id: "reasoning".into(),
            name: "Reasoning Module".into(),
            size_bytes: 4 * 1024 * 1024 * 1024,
            tasks: vec![AiTask::Summarize, AiTask::Generate, AiTask::Chat],
            model_path: None,
            installed: false,
        }
    }

    pub fn supports(&self, task: AiTask) -> bool {
        self.installed && self.tasks.contains(&task)
    }
}

#[derive(Debug, Default)]
pub struct AiModuleRegistry {
    modules: HashMap<String, AiModule>,
}

impl AiModuleRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::default();
        for module in [
            AiModule::grammar(),
            AiModule::translation(),
            AiModule::writing(),
            AiModule::reasoning(),
        ] {
            registry.modules.insert(module.id.clone(), module);
        }
        registry
    }

    pub fn list(&self) -> Vec<&AiModule> {
        self.modules.values().collect()
    }

    pub fn get(&self, id: &str) -> Option<&AiModule> {
        self.modules.get(id)
    }

    pub fn install(&mut self, id: &str, model_path: PathBuf) -> bool {
        if let Some(module) = self.modules.get_mut(id) {
            module.installed = true;
            module.model_path = Some(model_path);
            true
        } else {
            false
        }
    }

    pub fn smallest_for_task(&self, task: AiTask) -> Option<&AiModule> {
        self.modules
            .values()
            .filter(|m| m.supports(task))
            .min_by_key(|m| m.size_bytes)
    }
}
