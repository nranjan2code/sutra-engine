use crate::tcp_server::{LearnOptionsMsg, StorageRequest}; // Use internal types


/// Parse natural language commands into StorageRequest
pub struct NlParser;

impl NlParser {
    pub fn parse(text: &str) -> Option<StorageRequest> {
        let text = text.trim();
        let lower = text.to_lowercase();

        // "Remember that X" -> Learn
        if lower.starts_with("remember that") || lower.starts_with("learn that") {

            // TODO: Better slicing to handle "learn that" vs "remember that" length
            let content = if lower.starts_with("remember that") {
                text[13..].trim()
            } else {
                text[10..].trim()
            };

            return Some(StorageRequest::LearnConceptV2 {
                namespace: Some("default".to_string()),
                content: content.to_string(),
                options: LearnOptionsMsg::default(),
            });
        }

        // "Search for X" or "Query X"
        if lower.starts_with("search for") || lower.starts_with("find") {
            let query = if lower.starts_with("search for") {
                text[10..].trim()
            } else {
                text[4..].trim()
            };

            return Some(StorageRequest::QueryConcept {
                namespace: Some("default".to_string()),
                concept_id: query.to_string(), // QueryConcept uses query as ID approx
            });
        }
        
        // "Clear memory"
        if lower == "ls" || lower == "list" {
             return Some(StorageRequest::ListRecent {
                namespace: "default".to_string(),
                limit: 20,
            });
        }

        None
    }
}
