use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LocrawlConfig {
    ollama_url: String,
    default_model: String,
}

impl Default for LocrawlConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".into(),
            default_model: "llama3".into(),
        }
    }
}
