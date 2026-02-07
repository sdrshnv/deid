use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiEntity {
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub original: String,
}

pub fn detect_emails(text: &str) -> Vec<PiiEntity> {
    let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    re.find_iter(text)
        .map(|m| PiiEntity {
            entity_type: "email".to_string(),
            start: m.start(),
            end: m.end(),
            original: m.as_str().to_string(),
        })
        .collect()
}
