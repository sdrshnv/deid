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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::path::PathBuf;

    #[derive(Deserialize)]
    struct FakerEntry {
        #[serde(rename = "firstName")]
        first_name: String,
        #[serde(rename = "lastName")]
        last_name: String,
        email: String,
    }

    #[tokio::test]
    async fn test_redact_faker_entries() {
        // Read faker.json from project root (CARGO_MANIFEST_DIR is src-tauri/)
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let faker_path = manifest_dir.parent().unwrap().join("faker.json");
        let faker_json = std::fs::read_to_string(&faker_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", faker_path.display(), e));

        let entries: Vec<FakerEntry> = serde_json::from_str(&faker_json)
            .expect("Failed to parse faker.json");

        assert_eq!(entries.len(), 100, "Expected 100 faker entries");

        let mut missed_emails: Vec<String> = Vec::new();
        let mut missed_first_names: Vec<String> = Vec::new();
        let mut missed_last_names: Vec<String> = Vec::new();

        // Helper: serialize entries to pretty JSON
        fn entries_to_json(entries: &[&FakerEntry]) -> String {
            serde_json::to_string_pretty(
                &entries
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "firstName": e.first_name,
                            "lastName": e.last_name,
                            "email": e.email,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .expect("Failed to serialize batch")
        }

        // Helper: check which entries still have unredacted PII
        fn check_misses<'a>(
            redacted: &str,
            entries: &[&'a FakerEntry],
        ) -> Vec<&'a FakerEntry> {
            entries
                .iter()
                .filter(|e| {
                    redacted.contains(e.first_name.as_str())
                        || redacted.contains(e.last_name.as_str())
                        || redacted.contains(e.email.as_str())
                })
                .copied()
                .collect()
        }

        // Process in batches of 5
        for (batch_idx, batch) in entries.chunks(5).enumerate() {
            let batch_refs: Vec<&FakerEntry> = batch.iter().collect();
            let batch_json = entries_to_json(&batch_refs);

            let redacted = redact(&batch_json)
                .await
                .unwrap_or_else(|e| panic!("Batch {} redact() failed: {}", batch_idx, e));

            let mut still_missed = check_misses(&redacted, &batch_refs);

            // Retry with only the failed entries (up to 4 retries)
            for attempt in 2..=5 {
                if still_missed.is_empty() {
                    break;
                }
                eprintln!(
                    "Batch {}: attempt {} — retrying {} entries with missed names",
                    batch_idx,
                    attempt,
                    still_missed.len()
                );
                let retry_json = entries_to_json(&still_missed);
                let retry_redacted = redact(&retry_json)
                    .await
                    .unwrap_or_else(|e| panic!("Batch {} retry failed: {}", batch_idx, e));
                still_missed = check_misses(&retry_redacted, &still_missed);
            }

            for entry in still_missed {
                if redacted.contains(&entry.email) {
                    missed_emails.push(format!("batch {}: {}", batch_idx, entry.email));
                }
                if redacted.contains(&entry.first_name) {
                    missed_first_names.push(format!(
                        "batch {}: firstName={}",
                        batch_idx, entry.first_name
                    ));
                }
                if redacted.contains(&entry.last_name) {
                    missed_last_names.push(format!(
                        "batch {}: lastName={}",
                        batch_idx, entry.last_name
                    ));
                }
            }
        }

        // Report all failures
        if !missed_emails.is_empty() {
            eprintln!(
                "\nMISSED EMAILS ({}/{}):\n{}",
                missed_emails.len(),
                entries.len(),
                missed_emails.join("\n")
            );
        }
        if !missed_first_names.is_empty() {
            eprintln!(
                "\nMISSED FIRST NAMES ({}/{}):\n{}",
                missed_first_names.len(),
                entries.len(),
                missed_first_names.join("\n")
            );
        }
        if !missed_last_names.is_empty() {
            eprintln!(
                "\nMISSED LAST NAMES ({}/{}):\n{}",
                missed_last_names.len(),
                entries.len(),
                missed_last_names.join("\n")
            );
        }

        assert!(
            missed_emails.is_empty(),
            "{} emails were not redacted",
            missed_emails.len()
        );
        assert!(
            missed_first_names.is_empty(),
            "{} first names were not redacted",
            missed_first_names.len()
        );
        assert!(
            missed_last_names.is_empty(),
            "{} last names were not redacted",
            missed_last_names.len()
        );
    }
}
