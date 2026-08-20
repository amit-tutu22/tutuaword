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

    /// Mark modules available when cloud/local endpoints are configured (F28.S1).
    pub fn sync_endpoint_availability(
        &mut self,
        cloud_available: bool,
        local_available: bool,
    ) {
        use std::path::PathBuf;
        if local_available {
            self.install("grammar", PathBuf::from("local"));
            self.install("writing", PathBuf::from("local"));
        }
        if cloud_available {
            self.install("translation", PathBuf::from("cloud"));
            self.install("reasoning", PathBuf::from("cloud"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::AiTask;

    #[test]
    fn sync_marks_modules_when_endpoints_configured() {
        let mut registry = AiModuleRegistry::with_defaults();
        assert!(registry.get("grammar").unwrap().installed == false);
        registry.sync_endpoint_availability(true, true);
        assert!(registry.get("grammar").unwrap().installed);
        assert!(registry.get("translation").unwrap().installed);
        assert!(registry.smallest_for_task(AiTask::Grammar).is_some());
    }
}
