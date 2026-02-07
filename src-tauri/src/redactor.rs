use crate::ollama;
use crate::regex_engine::{self, PiiEntity};

pub async fn redact(text: &str) -> Result<String, String> {
    // Run regex detection (instant)
    let mut entities = regex_engine::detect_emails(text);

    // Run Ollama name detection (async, may fail)
    match ollama::detect_names(text).await {
        Ok(name_entities) => entities.extend(name_entities),
        Err(e) => {
            eprintln!("Ollama name detection unavailable: {}", e);
            // Continue with regex-only results
        }
    }

    if entities.is_empty() {
        return Ok(text.to_string());
    }

    // Sort by start offset ascending to process overlaps
    entities.sort_by_key(|e| e.start);

    // Remove overlapping entities: keep the first (leftmost) one
    let mut deduped: Vec<PiiEntity> = Vec::new();
    for entity in entities {
        if let Some(last) = deduped.last() {
            if entity.start < last.end {
                // Overlapping — skip
                continue;
            }
        }
        deduped.push(entity);
    }

    // Replace from end-to-start so byte offsets remain valid
    let mut result = text.to_string();
    for entity in deduped.iter().rev() {
        let replacement = format!("[REDACTED-{}]", entity.entity_type);
        result.replace_range(entity.start..entity.end, &replacement);
    }

    Ok(result)
}
