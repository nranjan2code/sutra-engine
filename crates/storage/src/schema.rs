//! Schema constants and validation for Sutra storage
//!
//! This module defines the schema for the conversation-first UI architecture,
//! including concept types, association types, and validation rules for metadata.
//!
//! ## Storage Architecture
//!
//! - **Domain Storage (domain-storage.dat):** Domain-specific knowledge concepts
//! - **User Storage (user-storage.dat):** Users, conversations, messages, organizations
//!
//! ## Multi-Tenancy
//!
//! All user storage concepts MUST have an organization_id for data isolation.

use std::collections::HashMap;

// ============================================================================
// Association Type Constants
// ============================================================================

/// Association types for linking concepts in the knowledge graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AssociationType {
    // Domain associations (original behavior)
    Semantic = 0,
    Causal = 1,
    Temporal = 2,
    Hierarchical = 3,
    Compositional = 4,

    // User/organization associations
    HasSession = 10,            // User -> Session
    BelongsToOrganization = 11, // User -> Organization
    HasRole = 12,               // User -> Role
    HasPermission = 13,         // Role -> Permission

    // Conversation associations
    OwnsConversation = 20, // User -> Conversation
    HasMessage = 21,       // Conversation -> Message
    AuthoredBy = 22,       // Message -> User
    InSpace = 23,          // Conversation -> Space

    // Cross-storage associations (links user storage to domain storage)
    UsesKnowledgeBase = 30, // Conversation -> DomainConcept (by storage name)

    // Audit trail
    TriggeredBy = 40, // AuditLog -> User
    RelatesTo = 41,   // AuditLog -> (any concept)
}

impl AssociationType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Semantic),
            1 => Some(Self::Causal),
            2 => Some(Self::Temporal),
            3 => Some(Self::Hierarchical),
            4 => Some(Self::Compositional),
            10 => Some(Self::HasSession),
            11 => Some(Self::BelongsToOrganization),
            12 => Some(Self::HasRole),
            13 => Some(Self::HasPermission),
            20 => Some(Self::OwnsConversation),
            21 => Some(Self::HasMessage),
            22 => Some(Self::AuthoredBy),
            23 => Some(Self::InSpace),
            30 => Some(Self::UsesKnowledgeBase),
            40 => Some(Self::TriggeredBy),
            41 => Some(Self::RelatesTo),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::Causal => "causal",
            Self::Temporal => "temporal",
            Self::Hierarchical => "hierarchical",
            Self::Compositional => "compositional",
            Self::HasSession => "has_session",
            Self::BelongsToOrganization => "belongs_to_organization",
            Self::HasRole => "has_role",
            Self::HasPermission => "has_permission",
            Self::OwnsConversation => "owns_conversation",
            Self::HasMessage => "has_message",
            Self::AuthoredBy => "authored_by",
            Self::InSpace => "in_space",
            Self::UsesKnowledgeBase => "uses_knowledge_base",
            Self::TriggeredBy => "triggered_by",
            Self::RelatesTo => "relates_to",
        }
    }
}

// ============================================================================
// Metadata Field Constants
// ============================================================================

/// Standard metadata field names for consistent querying
pub mod metadata_fields {
    pub const EMAIL: &str = "email";
    pub const PASSWORD_HASH: &str = "password_hash";
    pub const SALT: &str = "salt";
    pub const USERNAME: &str = "username";
    pub const FULL_NAME: &str = "full_name";

    pub const SESSION_TOKEN: &str = "session_token";
    pub const EXPIRES_AT: &str = "expires_at";
    pub const IP_ADDRESS: &str = "ip_address";
    pub const USER_AGENT: &str = "user_agent";

    pub const ORG_NAME: &str = "org_name";
    pub const ORG_DOMAIN: &str = "org_domain";
    pub const BILLING_TIER: &str = "billing_tier";
    pub const MAX_USERS: &str = "max_users";

    pub const CONVERSATION_TITLE: &str = "title";
    pub const CONVERSATION_STARRED: &str = "starred";
    pub const CONVERSATION_ARCHIVED: &str = "archived";
    pub const DOMAIN_STORAGE_NAME: &str = "domain_storage";

    pub const MESSAGE_ROLE: &str = "role"; // "user" or "assistant"
    pub const MESSAGE_CONFIDENCE: &str = "confidence";
    pub const MESSAGE_REASONING_PATHS: &str = "reasoning_paths";
    pub const MESSAGE_TOKEN_COUNT: &str = "token_count";

    pub const SPACE_COLOR: &str = "color";
    pub const SPACE_ICON: &str = "icon";
    pub const SPACE_DESCRIPTION: &str = "description";

    pub const PERMISSION_RESOURCE: &str = "resource";
    pub const PERMISSION_ACTION: &str = "action";

    pub const ROLE_NAME: &str = "role_name";
    pub const ROLE_LEVEL: &str = "level";

    pub const AUDIT_ACTION: &str = "action";
    pub const AUDIT_RESOURCE_TYPE: &str = "resource_type";
    pub const AUDIT_RESOURCE_ID: &str = "resource_id";
    pub const AUDIT_SUCCESS: &str = "success";
}

// ============================================================================
// Schema Validation Rules
// ============================================================================

/// Validate concept content and metadata based on concept type
pub struct SchemaValidator;

impl SchemaValidator {
    /// Validate required fields for a User concept
    pub fn validate_user(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::EMAIL)?;
        Self::require_field(attributes, metadata_fields::PASSWORD_HASH)?;
        Self::require_field(attributes, metadata_fields::SALT)?;

        // Validate email format (basic check)
        if let Some(email) = attributes.get(metadata_fields::EMAIL) {
            if !email.contains('@') || !email.contains('.') {
                return Err(format!("Invalid email format: {}", email));
            }
        }

        Ok(())
    }

    /// Validate required fields for a Session concept
    pub fn validate_session(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::SESSION_TOKEN)?;
        Self::require_field(attributes, metadata_fields::EXPIRES_AT)?;

        // Validate expiration timestamp is numeric
        if let Some(expires) = attributes.get(metadata_fields::EXPIRES_AT) {
            if expires.parse::<u64>().is_err() {
                return Err("expires_at must be a valid timestamp".to_string());
            }
        }

        Ok(())
    }

    /// Validate required fields for a Conversation concept
    pub fn validate_conversation(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::CONVERSATION_TITLE)?;

        // Domain storage name is optional but if present, validate format
        if let Some(storage) = attributes.get(metadata_fields::DOMAIN_STORAGE_NAME) {
            if storage.is_empty() || storage.len() > 256 {
                return Err("domain_storage must be 1-256 characters".to_string());
            }
        }

        Ok(())
    }

    /// Validate required fields for a Message concept
    pub fn validate_message(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::MESSAGE_ROLE)?;

        // Validate role is either "user" or "assistant"
        if let Some(role) = attributes.get(metadata_fields::MESSAGE_ROLE) {
            if role != "user" && role != "assistant" {
                return Err(format!(
                    "Invalid message role: {} (must be 'user' or 'assistant')",
                    role
                ));
            }
        }

        Ok(())
    }

    /// Validate required fields for a Space concept
    pub fn validate_space(_attributes: &HashMap<String, String>) -> Result<(), String> {
        // Spaces only need a name (in content), attributes are optional
        Ok(())
    }

    /// Validate required fields for an Organization concept
    pub fn validate_organization(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::ORG_NAME)?;
        Ok(())
    }

    /// Validate required fields for a Permission concept
    pub fn validate_permission(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::PERMISSION_RESOURCE)?;
        Self::require_field(attributes, metadata_fields::PERMISSION_ACTION)?;
        Ok(())
    }

    /// Validate required fields for a Role concept
    pub fn validate_role(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::ROLE_NAME)?;
        Ok(())
    }

    /// Validate required fields for an AuditLog concept
    pub fn validate_audit_log(attributes: &HashMap<String, String>) -> Result<(), String> {
        Self::require_field(attributes, metadata_fields::AUDIT_ACTION)?;
        Self::require_field(attributes, metadata_fields::AUDIT_RESOURCE_TYPE)?;
        Ok(())
    }

    // Helper to check if required field exists
    fn require_field(attributes: &HashMap<String, String>, field: &str) -> Result<(), String> {
        if !attributes.contains_key(field) {
            return Err(format!("Missing required field: {}", field));
        }
        if attributes.get(field).unwrap().is_empty() {
            return Err(format!("Required field cannot be empty: {}", field));
        }
        Ok(())
    }
}

// ============================================================================
// Content Templates
// ============================================================================

/// Generate standard content format for concepts
pub mod content_templates {

    /// Format user concept content
    pub fn format_user(email: &str, username: &str) -> String {
        format!("User: {} ({})", username, email)
    }

    /// Format session concept content
    pub fn format_session(user_id: &str, created_at: u64) -> String {
        format!("Session for user {} created at {}", user_id, created_at)
    }

    /// Format organization concept content
    pub fn format_organization(org_name: &str, org_id: &str) -> String {
        format!("Organization: {} (ID: {})", org_name, org_id)
    }

    /// Format conversation concept content
    pub fn format_conversation(title: &str, user_id: &str) -> String {
        format!("Conversation: '{}' (created by {})", title, user_id)
    }

    /// Format message concept content (just the message text)
    pub fn format_message(text: &str, role: &str) -> String {
        format!("[{}]: {}", role, text)
    }

    /// Format space concept content
    pub fn format_space(name: &str, description: &str) -> String {
        if description.is_empty() {
            format!("Space: {}", name)
        } else {
            format!("Space: {} - {}", name, description)
        }
    }

    /// Format audit log content
    pub fn format_audit_log(action: &str, resource_type: &str, resource_id: &str) -> String {
        format!("AUDIT: {} on {} {}", action, resource_type, resource_id)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_association_type_roundtrip() {
        let types = vec![
            AssociationType::Semantic,
            AssociationType::HasSession,
            AssociationType::OwnsConversation,
            AssociationType::UsesKnowledgeBase,
            AssociationType::TriggeredBy,
        ];

        for assoc_type in types {
            let value = assoc_type as u32;
            let restored = AssociationType::from_u32(value).unwrap();
            assert_eq!(assoc_type, restored);
        }
    }

    #[test]
    fn test_validate_user_success() {
        let mut attrs = HashMap::new();
        attrs.insert(
            metadata_fields::EMAIL.to_string(),
            "test@example.com".to_string(),
        );
        attrs.insert(
            metadata_fields::PASSWORD_HASH.to_string(),
            "hash123".to_string(),
        );
        attrs.insert(metadata_fields::SALT.to_string(), "salt456".to_string());

        assert!(SchemaValidator::validate_user(&attrs).is_ok());
    }

    #[test]
    fn test_validate_user_missing_email() {
        let mut attrs = HashMap::new();
        attrs.insert(
            metadata_fields::PASSWORD_HASH.to_string(),
            "hash123".to_string(),
        );
        attrs.insert(metadata_fields::SALT.to_string(), "salt456".to_string());

        assert!(SchemaValidator::validate_user(&attrs).is_err());
    }

    #[test]
    fn test_validate_user_invalid_email() {
        let mut attrs = HashMap::new();
        attrs.insert(
            metadata_fields::EMAIL.to_string(),
            "invalid-email".to_string(),
        );
        attrs.insert(
            metadata_fields::PASSWORD_HASH.to_string(),
            "hash123".to_string(),
        );
        attrs.insert(metadata_fields::SALT.to_string(), "salt456".to_string());

        assert!(SchemaValidator::validate_user(&attrs).is_err());
    }

    #[test]
    fn test_validate_session_success() {
        let mut attrs = HashMap::new();
        attrs.insert(
            metadata_fields::SESSION_TOKEN.to_string(),
            "token123".to_string(),
        );
        attrs.insert(
            metadata_fields::EXPIRES_AT.to_string(),
            "1234567890".to_string(),
        );

        assert!(SchemaValidator::validate_session(&attrs).is_ok());
    }

    #[test]
    fn test_validate_session_invalid_expiration() {
        let mut attrs = HashMap::new();
        attrs.insert(
            metadata_fields::SESSION_TOKEN.to_string(),
            "token123".to_string(),
        );
        attrs.insert(
            metadata_fields::EXPIRES_AT.to_string(),
            "not-a-number".to_string(),
        );

        assert!(SchemaValidator::validate_session(&attrs).is_err());
    }

    #[test]
    fn test_validate_message_valid_roles() {
        for role in &["user", "assistant"] {
            let mut attrs = HashMap::new();
            attrs.insert(metadata_fields::MESSAGE_ROLE.to_string(), role.to_string());
            assert!(SchemaValidator::validate_message(&attrs).is_ok());
        }
    }

    #[test]
    fn test_validate_message_invalid_role() {
        let mut attrs = HashMap::new();
        attrs.insert(
            metadata_fields::MESSAGE_ROLE.to_string(),
            "system".to_string(),
        );
        assert!(SchemaValidator::validate_message(&attrs).is_err());
    }

    #[test]
    fn test_content_templates() {
        let user = content_templates::format_user("alice@example.com", "alice");
        assert!(user.contains("alice"));
        assert!(user.contains("alice@example.com"));

        let conv = content_templates::format_conversation("My Chat", "user-123");
        assert!(conv.contains("My Chat"));
        assert!(conv.contains("user-123"));

        let msg = content_templates::format_message("Hello world", "user");
        assert!(msg.contains("Hello world"));
        assert!(msg.contains("user"));
    }
}
