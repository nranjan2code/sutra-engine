use crate::tcp_server::{LearnOptionsMsg, SemanticFilterMsg, StorageRequest}; // Use internal types

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

        // "ls" or "list"
        if lower == "ls" || lower == "list" {
            return Some(StorageRequest::ListRecent {
                namespace: "default".to_string(),
                limit: 20,
            });
        }

        // "status" or "engine status" -> GetAutonomyStats
        if lower == "status" || lower == "engine status" || lower == "autonomy status" {
            return Some(StorageRequest::GetAutonomyStats);
        }

        // "set goal: X" or "goal: X" -> CreateGoal
        if lower.starts_with("set goal:") || lower.starts_with("goal:") {
            let desc = if lower.starts_with("set goal:") {
                text[9..].trim()
            } else {
                text[5..].trim()
            };

            return Some(StorageRequest::CreateGoal {
                namespace: Some("default".to_string()),
                description: desc.to_string(),
                condition: desc.to_string(), // Use description as condition text
                action: format!("notify: {}", desc),
                priority: 5,
            });
        }

        // "list goals" or "goals"
        if lower == "list goals" || lower == "goals" {
            return Some(StorageRequest::ListGoals {
                namespace: Some("default".to_string()),
            });
        }

        // "subscribe to X" or "watch for X" -> Subscribe
        if lower.starts_with("subscribe to") || lower.starts_with("watch for") {
            let term = if lower.starts_with("subscribe to") {
                text[12..].trim()
            } else {
                text[9..].trim()
            };

            return Some(StorageRequest::Subscribe {
                filter: SemanticFilterMsg {
                    required_terms: vec![term.to_string()],
                    ..Default::default()
                },
                callback_addr: String::new(), // Log-only mode
            });
        }

        None
    }
}
