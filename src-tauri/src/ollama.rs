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
    /// Qwen3 (a thinking model) puts structured output in this field instead of `response`.
    thinking: Option<String>,
}

#[derive(Deserialize)]
struct NamesResult {
    names: Vec<String>,
}

pub async fn detect_names(text: &str) -> Result<Vec<PiiEntity>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
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
        model: "qwen3:4b".to_string(),
        prompt: format!(
            "Extract every person name from the text below.\n\n\
             Instructions:\n\
             - Return each first name and each last name as separate items.\n\
             - If the text contains labeled fields (e.g. \"firstName\", \"lastName\", \"name\", \"surname\"), \
               treat every value associated with those fields as a person name regardless of how unusual it looks.\n\
             - Include hyphenated names (e.g. \"Smith-Jones\") and names with apostrophes (e.g. \"O'Brien\") as complete single names.\n\
             - Be thorough — do not skip unusual, uncommon, or non-English names. If a value is labeled as a name, it IS a name.\n\n\
             Text: {}",
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

    let json_str = if gen_response.response.is_empty() {
        gen_response.thinking.as_deref().unwrap_or("")
    } else {
        &gen_response.response
    };

    let names_result: NamesResult = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse names JSON: {}", e))?;

    let mut entities = Vec::new();
    for name in &names_result.names {
        if name.is_empty() {
            continue;
        }
        // Find all occurrences of this name in the original text
        let mut found = false;
        let mut search_start = 0;
        while let Some(pos) = text[search_start..].find(name.as_str()) {
            found = true;
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

        // If the full name wasn't found (e.g. LLM returned "Dasia Turner" but
        // the text has firstName/lastName in separate JSON fields), search for
        // each whitespace-delimited part individually.
        if !found {
            let parts: Vec<&str> = name.split_whitespace().collect();
            if parts.len() > 1 {
                for part in parts {
                    let mut search_start = 0;
                    while let Some(pos) = text[search_start..].find(part) {
                        let start = search_start + pos;
                        let end = start + part.len();
                        entities.push(PiiEntity {
                            entity_type: "name".to_string(),
                            start,
                            end,
                            original: part.to_string(),
                        });
                        search_start = end;
                    }
                }
            }
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
