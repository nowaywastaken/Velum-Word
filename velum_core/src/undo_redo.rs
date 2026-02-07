//! Undo/Redo system for the document editor.
//!
//! This module provides a command-based undo/redo system with support for:
//! - Text insertion/deletion commands
//! - Command merging for continuous input

use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::piece_tree::{PieceTree, Selection};

/// Default maximum history size
pub const DEFAULT_MAX_HISTORY_SIZE: usize = 100;

/// Default time window for merging commands (500ms)
pub const DEFAULT_MERGE_WINDOW_MS: u64 = 500;

/// Error types for command execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    ExecutionFailed(String),
    InvalidState(String),
    UnsupportedOperation(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            CommandError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            CommandError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// Trait for all document commands
pub trait Command: Send + Sync + std::fmt::Debug {
    /// Executes the command on the document
    fn execute(&self, doc: &mut PieceTree) -> Result<CommandExecution, CommandError>;

    /// Undoes the command
    fn undo(&self, doc: &mut PieceTree, execution: &CommandExecution) -> Result<(), CommandError>;

    /// Redoes the command
    fn redo(&self, doc: &mut PieceTree, execution: &CommandExecution) -> Result<(), CommandError>;

    /// Attempts to merge with another command
    fn merge(&self, other: &dyn Command) -> Option<Arc<dyn Command>>;

    /// Returns whether this command can be merged with subsequent commands
    fn is_mergeable(&self) -> bool;

    /// Returns a human-readable name for this command
    fn name(&self) -> &str;

    /// Returns self as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Execution state captured during command execution
#[derive(Debug, Clone)]
pub struct CommandExecution {
    /// The type of operation performed
    pub operation_type: OperationType,
    /// Position in the document (byte offset)
    pub offset: usize,
    /// Length of the affected text
    pub length: usize,
    /// The text that was inserted (for insert operations)
    pub inserted_text: Option<String>,
    /// The text that was deleted (for delete operations)
    pub deleted_text: Option<String>,
    /// Selection state before the command
    pub prev_selection: Selection,
    /// Selection state after the command
    pub next_selection: Selection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationType {
    Insert,
    Delete,
    Composite,
    Custom,
}

/// Metadata for a command in the history
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Timestamp when the command was executed
    pub timestamp: Instant,
    /// User-friendly name
    pub display_name: String,
    /// Whether this command was created by merging multiple commands
    pub is_merged: bool,
}

impl CommandMetadata {
    pub fn new(display_name: impl Into<String>) -> Self {
        CommandMetadata {
            timestamp: Instant::now(),
            display_name: display_name.into(),
            is_merged: false,
        }
    }

    pub fn merged() -> Self {
        CommandMetadata {
            timestamp: Instant::now(),
            display_name: "Merged operations".to_string(),
            is_merged: true,
        }
    }
}

/// Wrapper that combines a command with its execution state and metadata
#[derive(Clone)]
pub struct CommandRecord {
    pub command: Arc<dyn Command>,
    pub execution: CommandExecution,
    pub metadata: CommandMetadata,
}

// ==================== Insert Command ====================

/// Command for inserting text
#[derive(Debug, Clone)]
pub struct InsertCommand {
    offset: usize,
    text: String,
}

impl InsertCommand {
    pub fn new(offset: usize, text: impl Into<String>) -> Self {
        InsertCommand {
            offset,
            text: text.into(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn length(&self) -> usize {
        self.text.len()
    }
}

impl Command for InsertCommand {
    fn execute(&self, doc: &mut PieceTree) -> Result<CommandExecution, CommandError> {
        let prev_selection = doc.selection;
        doc.insert(self.offset, self.text.clone())
            .then(|| ())
            .ok_or_else(|| CommandError::ExecutionFailed("Insert failed".to_string()))?;

        Ok(CommandExecution {
            operation_type: OperationType::Insert,
            offset: self.offset,
            length: self.text.len(),
            inserted_text: Some(self.text.clone()),
            deleted_text: None,
            prev_selection,
            next_selection: doc.selection,
        })
    }

    fn undo(&self, doc: &mut PieceTree, execution: &CommandExecution) -> Result<(), CommandError> {
        doc.delete(execution.offset, execution.length)
            .then(|| ())
            .ok_or_else(|| CommandError::ExecutionFailed("Undo insert failed".to_string()))?;
        doc.selection = execution.prev_selection;
        Ok(())
    }

    fn redo(&self, doc: &mut PieceTree, execution: &CommandExecution) -> Result<(), CommandError> {
        if let Some(ref text) = execution.inserted_text {
            doc.insert(execution.offset, text.clone())
                .then(|| ())
                .ok_or_else(|| CommandError::ExecutionFailed("Redo insert failed".to_string()))?;
            doc.selection = execution.next_selection;
        }
        Ok(())
    }

    fn merge(&self, other: &dyn Command) -> Option<Arc<dyn Command>> {
        if let Some(other_insert) = other.as_any().downcast_ref::<InsertCommand>() {
            // Merge if other is immediately after this (continuous typing)
            if other_insert.offset == self.offset + self.text.len() {
                let mut new_text = self.text.clone();
                new_text.push_str(&other_insert.text);
                return Some(Arc::new(InsertCommand {
                    offset: self.offset,
                    text: new_text,
                }));
            } else if other_insert.offset == self.offset && !other_insert.text.is_empty() {
                // Same position insert (e.g., IME commit)
                let mut new_text = self.text.clone();
                new_text.push_str(&other_insert.text);
                return Some(Arc::new(InsertCommand {
                    offset: self.offset,
                    text: new_text,
                }));
            }
        }
        None
    }

    fn is_mergeable(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "Insert"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ==================== Delete Command ====================

/// Command for deleting text
#[derive(Debug, Clone)]
pub struct DeleteCommand {
    offset: usize,
    length: usize,
}

impl DeleteCommand {
    pub fn new(offset: usize, length: usize) -> Self {
        DeleteCommand {
            offset,
            length,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl Command for DeleteCommand {
    fn execute(&self, doc: &mut PieceTree) -> Result<CommandExecution, CommandError> {
        let prev_selection = doc.selection;
        let deleted_text = doc.get_text_range(self.offset, self.length);
        doc.delete(self.offset, self.length)
            .then(|| ())
            .ok_or_else(|| CommandError::ExecutionFailed("Delete failed".to_string()))?;

        Ok(CommandExecution {
            operation_type: OperationType::Delete,
            offset: self.offset,
            length: self.length,
            inserted_text: None,
            deleted_text: Some(deleted_text),
            prev_selection,
            next_selection: doc.selection,
        })
    }

    fn undo(&self, doc: &mut PieceTree, execution: &CommandExecution) -> Result<(), CommandError> {
        if let Some(ref text) = execution.deleted_text {
            doc.insert(execution.offset, text.clone())
                .then(|| ())
                .ok_or_else(|| CommandError::ExecutionFailed("Undo delete failed".to_string()))?;
        }
        doc.selection = execution.prev_selection;
        Ok(())
    }

    fn redo(&self, doc: &mut PieceTree, execution: &CommandExecution) -> Result<(), CommandError> {
        doc.delete(execution.offset, execution.length)
            .then(|| ())
            .ok_or_else(|| CommandError::ExecutionFailed("Redo delete failed".to_string()))?;
        doc.selection = execution.prev_selection;
        Ok(())
    }

    fn merge(&self, _other: &dyn Command) -> Option<Arc<dyn Command>> {
        None
    }

    fn is_mergeable(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Delete"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ==================== Composite Command ====================

/// A composite command that wraps multiple commands
#[derive(Debug, Clone)]
pub struct CompositeCommand {
    name: String,
    commands: Vec<Arc<dyn Command>>,
}

impl CompositeCommand {
    pub fn new(name: impl Into<String>) -> Self {
        CompositeCommand {
            name: name.into(),
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: Arc<dyn Command>) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[Arc<dyn Command>] {
        &self.commands
    }
}

impl Command for CompositeCommand {
    fn execute(&self, _doc: &mut PieceTree) -> Result<CommandExecution, CommandError> {
        Ok(CommandExecution {
            operation_type: OperationType::Composite,
            offset: 0,
            length: 0,
            inserted_text: None,
            deleted_text: None,
            prev_selection: Selection::default(),
            next_selection: Selection::default(),
        })
    }

    fn undo(&self, _doc: &mut PieceTree, _execution: &CommandExecution) -> Result<(), CommandError> {
        for _cmd in self.commands.iter().rev() {
            // Would need to track execution for each command
        }
        Ok(())
    }

    fn redo(&self, _doc: &mut PieceTree, _execution: &CommandExecution) -> Result<(), CommandError> {
        for _cmd in &self.commands {
            // Would need to track execution for each command
        }
        Ok(())
    }

    fn merge(&self, _other: &dyn Command) -> Option<Arc<dyn Command>> {
        None
    }

    fn is_mergeable(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ==================== Undo/Redo Manager ====================

pub struct UndoRedoManager {
    undo_stack: Vec<CommandRecord>,
    redo_stack: Vec<CommandRecord>,
    max_history_size: usize,
    merge_window_ms: u64,
    last_command_time: Option<Instant>,
}

impl Default for UndoRedoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for UndoRedoManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UndoRedoManager")
            .field("undo_stack_len", &self.undo_stack.len())
            .field("redo_stack_len", &self.redo_stack.len())
            .field("max_history_size", &self.max_history_size)
            .finish()
    }
}

impl UndoRedoManager {
    pub fn new() -> Self {
        UndoRedoManager {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_size: DEFAULT_MAX_HISTORY_SIZE,
            merge_window_ms: DEFAULT_MERGE_WINDOW_MS,
            last_command_time: None,
        }
    }

    pub fn with_settings(max_history_size: usize, merge_window_ms: u64) -> Self {
        UndoRedoManager {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_size,
            merge_window_ms,
            last_command_time: None,
        }
    }

    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        while self.undo_stack.len() > self.max_history_size {
            self.undo_stack.remove(0);
        }
    }

    pub fn set_merge_window(&mut self, duration: Duration) {
        self.merge_window_ms = duration.as_millis() as u64;
    }

    pub fn execute(&mut self, doc: &mut PieceTree, command: Arc<dyn Command>) -> Result<(), CommandError> {
        let execution = command.execute(doc)?;

        let should_merge = command.is_mergeable()
            && self.last_command_time.map_or(false, |t| {
                t.elapsed() < Duration::from_millis(self.merge_window_ms)
            })
            && !self.undo_stack.is_empty();

        self.redo_stack.clear();

        if should_merge {
            let last_record = self.undo_stack.last_mut().unwrap();
            if let Some(merged) = last_record.command.merge(&*command) {
                let new_execution = merged.execute(doc)?;
                last_record.command = merged;
                last_record.execution = new_execution;
                last_record.metadata = CommandMetadata::merged();
            } else {
                self.push_command(command, execution);
            }
        } else {
            self.push_command(command, execution);
        }

        self.last_command_time = Some(Instant::now());

        Ok(())
    }

    fn push_command(&mut self, command: Arc<dyn Command>, execution: CommandExecution) {
        if self.undo_stack.len() >= self.max_history_size {
            self.undo_stack.remove(0);
        }

        self.undo_stack.push(CommandRecord {
            command,
            execution,
            metadata: CommandMetadata::new(""),
        });
    }

    pub fn undo(&mut self, doc: &mut PieceTree) -> Result<(), CommandError> {
        if let Some(record) = self.undo_stack.pop() {
            record.command.undo(doc, &record.execution)?;
            self.redo_stack.push(record);
            Ok(())
        } else {
            Err(CommandError::InvalidState("Nothing to undo".to_string()))
        }
    }

    pub fn redo(&mut self, doc: &mut PieceTree) -> Result<(), CommandError> {
        if let Some(record) = self.redo_stack.pop() {
            record.command.redo(doc, &record.execution)?;
            self.undo_stack.push(record);
            Ok(())
        } else {
            Err(CommandError::InvalidState("Nothing to redo".to_string()))
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn next_undo_name(&self) -> Option<&str> {
        self.undo_stack.last().map(|r| r.command.name())
    }

    pub fn next_redo_name(&self) -> Option<&str> {
        self.redo_stack.last().map(|r| r.command.name())
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.last_command_time = None;
    }

    #[cfg(test)]
    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    #[cfg(test)]
    pub fn redo_stack_len(&self) -> usize {
        self.redo_stack.len()
    }
}

// ==================== Unit Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_manager() {
        let manager = UndoRedoManager::new();
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
    }

    #[test]
    fn test_manager_with_settings() {
        let manager = UndoRedoManager::with_settings(50, 1000);
        assert_eq!(manager.undo_stack_len(), 0);
        assert_eq!(manager.redo_stack_len(), 0);
    }

    #[test]
    fn test_set_max_history_size() {
        let mut manager = UndoRedoManager::with_settings(3, 0);
        assert_eq!(manager.max_history_size, 3);

        manager.set_max_history_size(10);
        assert_eq!(manager.max_history_size, 10);
    }

    #[test]
    fn test_clear() {
        let mut manager = UndoRedoManager::new();
        manager.clear();
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_command_metadata_new() {
        let metadata = CommandMetadata::new("Test");
        assert_eq!(metadata.display_name, "Test");
        assert!(!metadata.is_merged);
    }

    #[test]
    fn test_command_metadata_merged() {
        let metadata = CommandMetadata::merged();
        assert_eq!(metadata.display_name, "Merged operations");
        assert!(metadata.is_merged);
    }

    #[test]
    fn test_operation_types() {
        assert_eq!(OperationType::Insert, OperationType::Insert);
        assert_eq!(OperationType::Delete, OperationType::Delete);
        assert_ne!(OperationType::Insert, OperationType::Delete);
    }

    #[test]
    fn test_insert_command_execute() {
        let mut pt = PieceTree::new("World".to_string());
        let cmd = InsertCommand::new(0, "Hello ".to_string());

        let execution = cmd.execute(&mut pt).unwrap();

        assert_eq!(execution.operation_type, OperationType::Insert);
        assert_eq!(execution.offset, 0);
        assert_eq!(execution.length, 6);
        assert_eq!(execution.inserted_text, Some("Hello ".to_string()));
        assert_eq!(pt.get_text(), "Hello World");
    }

    #[test]
    fn test_insert_command_undo() {
        let mut pt = PieceTree::new("World".to_string());
        let cmd = InsertCommand::new(0, "Hello ".to_string());

        // Execute insert
        let execution = cmd.execute(&mut pt).unwrap();
        assert_eq!(pt.get_text(), "Hello World");

        // Undo insert
        cmd.undo(&mut pt, &execution).unwrap();
        assert_eq!(pt.get_text(), "World");
    }

    #[test]
    fn test_insert_command_redo() {
        let mut pt = PieceTree::new("World".to_string());
        let cmd = InsertCommand::new(0, "Hello ".to_string());

        // Execute insert
        let execution = cmd.execute(&mut pt).unwrap();
        assert_eq!(pt.get_text(), "Hello World");

        // Undo
        cmd.undo(&mut pt, &execution).unwrap();
        assert_eq!(pt.get_text(), "World");

        // Redo
        cmd.redo(&mut pt, &execution).unwrap();
        assert_eq!(pt.get_text(), "Hello World");
    }

    #[test]
    fn test_delete_command_execute() {
        let mut pt = PieceTree::new("Hello World".to_string());
        let cmd = DeleteCommand::new(5, 6);

        let execution = cmd.execute(&mut pt).unwrap();

        assert_eq!(execution.operation_type, OperationType::Delete);
        assert_eq!(execution.offset, 5);
        assert_eq!(execution.length, 6);
        assert_eq!(execution.deleted_text, Some(" World".to_string()));
        assert_eq!(pt.get_text(), "Hello");
    }

    #[test]
    fn test_delete_command_undo() {
        let mut pt = PieceTree::new("Hello World".to_string());
        let cmd = DeleteCommand::new(5, 6);

        // Execute delete
        let execution = cmd.execute(&mut pt).unwrap();
        assert_eq!(pt.get_text(), "Hello");

        // Undo delete
        cmd.undo(&mut pt, &execution).unwrap();
        assert_eq!(pt.get_text(), "Hello World");
    }

    #[test]
    fn test_delete_command_redo() {
        let mut pt = PieceTree::new("Hello World".to_string());
        let cmd = DeleteCommand::new(5, 6);

        // Execute delete
        let execution = cmd.execute(&mut pt).unwrap();
        assert_eq!(pt.get_text(), "Hello");

        // Undo
        cmd.undo(&mut pt, &execution).unwrap();
        assert_eq!(pt.get_text(), "Hello World");

        // Redo
        cmd.redo(&mut pt, &execution).unwrap();
        assert_eq!(pt.get_text(), "Hello");
    }

    #[test]
    fn test_insert_command_merge_adjacent() {
        let cmd1 = InsertCommand::new(0, "Hello".to_string());
        let cmd2 = InsertCommand::new(5, " World".to_string());

        let merged = cmd1.merge(&cmd2);
        assert!(merged.is_some());

        let merged = merged.unwrap();
        if let Some(insert) = merged.as_any().downcast_ref::<InsertCommand>() {
            assert_eq!(insert.text(), "Hello World");
            assert_eq!(insert.offset(), 0);
        }
    }

    #[test]
    fn test_insert_command_merge_same_position() {
        let cmd1 = InsertCommand::new(0, "H".to_string());
        let cmd2 = InsertCommand::new(0, "ello".to_string());

        let merged = cmd1.merge(&cmd2);
        assert!(merged.is_some());

        let merged = merged.unwrap();
        if let Some(insert) = merged.as_any().downcast_ref::<InsertCommand>() {
            assert_eq!(insert.text(), "Hello");
            assert_eq!(insert.offset(), 0);
        }
    }

    #[test]
    fn test_insert_command_no_merge_different_position() {
        let cmd1 = InsertCommand::new(0, "Hello".to_string());
        let cmd2 = InsertCommand::new(10, " World".to_string());

        let merged = cmd1.merge(&cmd2);
        assert!(merged.is_none());
    }

    #[test]
    fn test_command_names() {
        let insert = InsertCommand::new(0, "test",);
        assert_eq!(insert.name(), "Insert");

        let delete = DeleteCommand::new(0, 5);
        assert_eq!(delete.name(), "Delete");
    }

    #[test]
    fn test_insert_command_properties() {
        let cmd = InsertCommand::new(5, "Hello");
        assert_eq!(cmd.offset(), 5);
        assert_eq!(cmd.text(), "Hello");
        assert_eq!(cmd.length(), 5);
        assert!(cmd.is_mergeable());
    }

    #[test]
    fn test_delete_command_properties() {
        let cmd = DeleteCommand::new(10, 5);
        assert_eq!(cmd.offset(), 10);
        assert_eq!(cmd.length(), 5);
        assert!(!cmd.is_mergeable());
    }

    #[test]
    fn test_command_execution_structure() {
        let execution = CommandExecution {
            operation_type: OperationType::Insert,
            offset: 10,
            length: 5,
            inserted_text: Some("hello".to_string()),
            deleted_text: None,
            prev_selection: Selection::new(0, 0),
            next_selection: Selection::new(5, 5),
        };

        assert_eq!(execution.operation_type, OperationType::Insert);
        assert_eq!(execution.offset, 10);
        assert_eq!(execution.length, 5);
        assert_eq!(execution.inserted_text, Some("hello".to_string()));
    }

    #[test]
    fn test_undo_redo_counts() {
        let mut manager = UndoRedoManager::new();
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
    }

    #[test]
    fn test_merge_window_default() {
        let manager = UndoRedoManager::new();
        assert_eq!(manager.merge_window_ms, DEFAULT_MERGE_WINDOW_MS);
    }

    #[test]
    fn test_max_history_default() {
        let manager = UndoRedoManager::new();
        assert_eq!(manager.max_history_size, DEFAULT_MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_next_undo_redo_names_empty() {
        let manager = UndoRedoManager::new();
        assert!(manager.next_undo_name().is_none());
        assert!(manager.next_redo_name().is_none());
    }

    #[test]
    fn test_composite_command() {
        let mut composite = CompositeCommand::new("Test Composite");
        assert_eq!(composite.name(), "Test Composite");
        assert!(!composite.is_mergeable());
    }

    #[test]
    fn test_piece_tree_undo_redo_integration() {
        let mut pt = PieceTree::new("".to_string());

        // Insert "Hello"
        pt.insert(0, "Hello".to_string());
        assert_eq!(pt.get_text(), "Hello");

        // Insert " World"
        pt.insert(5, " World".to_string());
        assert_eq!(pt.get_text(), "Hello World");

        // Delete " World"
        pt.delete(5, 6);
        assert_eq!(pt.get_text(), "Hello");

        // Should be able to undo delete
        assert!(pt.can_undo());
    }
}
