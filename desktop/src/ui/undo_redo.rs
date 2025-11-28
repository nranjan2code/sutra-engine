//! Global Undo/Redo System
//!
//! A command-based undo/redo stack for all destructive operations.
//! Supports arbitrary commands with execute/undo operations.

use std::collections::VecDeque;
use chrono::{DateTime, Local};
use sutra_storage::ConceptId;

/// Maximum number of commands to keep in history
const MAX_HISTORY: usize = 100;

/// Global command history for undo/redo
pub struct CommandHistory {
    /// Stack of executed commands (for undo)
    undo_stack: VecDeque<Command>,
    /// Stack of undone commands (for redo)
    redo_stack: VecDeque<Command>,
    /// Whether history is enabled
    pub enabled: bool,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(MAX_HISTORY),
            redo_stack: VecDeque::with_capacity(MAX_HISTORY),
            enabled: true,
        }
    }
}

impl CommandHistory {
    /// Create a new command history
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Push a command onto the undo stack
    pub fn push(&mut self, command: Command) {
        if !self.enabled {
            return;
        }
        
        // Clear redo stack when new command is added
        self.redo_stack.clear();
        
        // Add to undo stack
        self.undo_stack.push_back(command);
        
        // Trim if over capacity
        while self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.pop_front();
        }
    }
    
    /// Pop the last command for undo
    pub fn pop_undo(&mut self) -> Option<Command> {
        let cmd = self.undo_stack.pop_back()?;
        self.redo_stack.push_back(cmd.clone());
        Some(cmd)
    }
    
    /// Pop from redo stack
    pub fn pop_redo(&mut self) -> Option<Command> {
        let cmd = self.redo_stack.pop_back()?;
        self.undo_stack.push_back(cmd.clone());
        Some(cmd)
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    /// Get the number of undoable commands
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }
    
    /// Get the number of redoable commands
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
    
    /// Get description of next undo action
    pub fn next_undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(|c| c.description.as_str())
    }
    
    /// Get description of next redo action
    pub fn next_redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|c| c.description.as_str())
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
    
    /// Get recent commands (for display)
    pub fn recent_commands(&self, limit: usize) -> Vec<&Command> {
        self.undo_stack.iter().rev().take(limit).collect()
    }
}

/// A command that can be undone/redone
#[derive(Debug, Clone)]
pub struct Command {
    /// Type of command
    pub command_type: CommandType,
    /// Human-readable description
    pub description: String,
    /// When the command was executed
    pub timestamp: DateTime<Local>,
    /// Data needed for undo/redo
    pub data: CommandData,
}

impl Command {
    /// Create a new learn command
    pub fn learn(content: String, concept_id: ConceptId, confidence: f32) -> Self {
        Self {
            command_type: CommandType::Learn,
            description: format!("Learn: \"{}\"", truncate(&content, 30)),
            timestamp: Local::now(),
            data: CommandData::Learn {
                content,
                concept_id,
                confidence,
            },
        }
    }
    
    /// Create a new delete command
    pub fn delete(content: String, concept_id: ConceptId, confidence: f32, strength: f32, neighbors: Vec<String>) -> Self {
        Self {
            command_type: CommandType::Delete,
            description: format!("Delete: \"{}\"", truncate(&content, 30)),
            timestamp: Local::now(),
            data: CommandData::Delete {
                content,
                concept_id,
                confidence,
                strength,
                neighbors,
            },
        }
    }
    
    /// Create a batch learn command
    pub fn batch_learn(count: usize, concepts: Vec<(String, ConceptId)>) -> Self {
        Self {
            command_type: CommandType::BatchLearn,
            description: format!("Batch learn {} concepts", count),
            timestamp: Local::now(),
            data: CommandData::BatchLearn { concepts },
        }
    }
    
    /// Create a clear all command
    pub fn clear_all(backup: Vec<ConceptBackup>) -> Self {
        Self {
            command_type: CommandType::ClearAll,
            description: format!("Clear all ({} concepts)", backup.len()),
            timestamp: Local::now(),
            data: CommandData::ClearAll { backup },
        }
    }
    
    /// Create an import command
    pub fn import(count: usize, concept_ids: Vec<ConceptId>) -> Self {
        Self {
            command_type: CommandType::Import,
            description: format!("Import {} concepts", count),
            timestamp: Local::now(),
            data: CommandData::Import { concept_ids },
        }
    }
    
    /// Get the icon for this command type
    pub fn icon(&self) -> &'static str {
        match self.command_type {
            CommandType::Learn => "📝",
            CommandType::Delete => "🗑️",
            CommandType::BatchLearn => "📚",
            CommandType::ClearAll => "🧹",
            CommandType::Import => "📥",
        }
    }
}

/// Types of undoable commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    /// Learn a single concept
    Learn,
    /// Delete a single concept
    Delete,
    /// Learn multiple concepts
    BatchLearn,
    /// Clear all data
    ClearAll,
    /// Import data
    Import,
}

/// Data associated with a command for undo/redo
#[derive(Debug, Clone)]
pub enum CommandData {
    /// Data for learn command
    Learn {
        content: String,
        concept_id: ConceptId,
        confidence: f32,
    },
    /// Data for delete command (stores full backup for restore)
    Delete {
        content: String,
        concept_id: ConceptId,
        confidence: f32,
        strength: f32,
        neighbors: Vec<String>,
    },
    /// Data for batch learn
    BatchLearn {
        concepts: Vec<(String, ConceptId)>,
    },
    /// Data for clear all (full backup)
    ClearAll {
        backup: Vec<ConceptBackup>,
    },
    /// Data for import
    Import {
        concept_ids: Vec<ConceptId>,
    },
}

/// Full backup of a concept for restore
#[derive(Debug, Clone)]
pub struct ConceptBackup {
    pub content: String,
    pub concept_id: ConceptId,
    pub confidence: f32,
    pub strength: f32,
    pub neighbors: Vec<String>,
}

/// Truncate a string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// Result of an undo/redo operation
#[derive(Debug, Clone)]
pub enum UndoRedoResult {
    /// Successfully undid a learn (need to delete)
    UndoLearn { concept_id: ConceptId },
    /// Successfully undid a delete (need to restore)
    UndoDelete { 
        content: String, 
        confidence: f32,
        strength: f32,
    },
    /// Successfully undid a batch learn
    UndoBatchLearn { concept_ids: Vec<ConceptId> },
    /// Successfully undid a clear all (need to restore)
    UndoClearAll { backup: Vec<ConceptBackup> },
    /// Successfully undid an import
    UndoImport { concept_ids: Vec<ConceptId> },
    /// Redo learn
    RedoLearn { content: String, confidence: f32 },
    /// Redo delete
    RedoDelete { concept_id: ConceptId },
    /// Redo batch learn
    RedoBatchLearn { contents: Vec<String> },
    /// Redo clear all
    RedoClearAll,
    /// Redo import (not supported - would need file)
    RedoImportNotSupported,
}

impl CommandHistory {
    /// Get the operation needed to undo the last command
    pub fn get_undo_operation(&self) -> Option<UndoRedoResult> {
        let cmd = self.undo_stack.back()?;
        Some(match &cmd.data {
            CommandData::Learn { concept_id, .. } => {
                UndoRedoResult::UndoLearn { concept_id: *concept_id }
            }
            CommandData::Delete { content, confidence, strength, .. } => {
                UndoRedoResult::UndoDelete { 
                    content: content.clone(), 
                    confidence: *confidence,
                    strength: *strength,
                }
            }
            CommandData::BatchLearn { concepts } => {
                UndoRedoResult::UndoBatchLearn { 
                    concept_ids: concepts.iter().map(|(_, id)| *id).collect() 
                }
            }
            CommandData::ClearAll { backup } => {
                UndoRedoResult::UndoClearAll { backup: backup.clone() }
            }
            CommandData::Import { concept_ids } => {
                UndoRedoResult::UndoImport { concept_ids: concept_ids.clone() }
            }
        })
    }
    
    /// Get the operation needed to redo the last undone command
    pub fn get_redo_operation(&self) -> Option<UndoRedoResult> {
        let cmd = self.redo_stack.back()?;
        Some(match &cmd.data {
            CommandData::Learn { content, confidence, .. } => {
                UndoRedoResult::RedoLearn { content: content.clone(), confidence: *confidence }
            }
            CommandData::Delete { concept_id, .. } => {
                UndoRedoResult::RedoDelete { concept_id: *concept_id }
            }
            CommandData::BatchLearn { concepts } => {
                UndoRedoResult::RedoBatchLearn { 
                    contents: concepts.iter().map(|(c, _)| c.clone()).collect() 
                }
            }
            CommandData::ClearAll { .. } => {
                UndoRedoResult::RedoClearAll
            }
            CommandData::Import { .. } => {
                UndoRedoResult::RedoImportNotSupported
            }
        })
    }
}
