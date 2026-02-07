use crate::regex_engine::PiiEntity;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: serde_json::Value,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Deserialize)]
struct NamesResult {
    names: Vec<String>,
}

pub async fn detect_names(text: &str) -> Result<Vec<PiiEntity>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "names": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["names"]
    });

    let request = GenerateRequest {
        model: "llama3.2:3b".to_string(),
        prompt: format!(
            "List every person name (first, last, or full) that appears in the following text. Return ONLY the names as a JSON object.\n\nText: {}",
            text
        ),
        stream: false,
        format: schema,
    };

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    let gen_response: GenerateResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    let names_result: NamesResult = serde_json::from_str(&gen_response.response)
        .map_err(|e| format!("Failed to parse names JSON: {}", e))?;

    let mut entities = Vec::new();
    for name in &names_result.names {
        if name.is_empty() {
            continue;
        }
        // Find all occurrences of this name in the original text
        let mut search_start = 0;
        while let Some(pos) = text[search_start..].find(name.as_str()) {
            let start = search_start + pos;
            let end = start + name.len();
            entities.push(PiiEntity {
                entity_type: "name".to_string(),
                start,
                end,
                original: name.clone(),
            });
            search_start = end;
        }
    }

    Ok(entities)
}

pub async fn check_connection() -> bool {
    let client = match Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    client
        .get("http://localhost:11434/")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
