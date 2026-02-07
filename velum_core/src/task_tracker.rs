// Task Tracker System for Velum-Word
// Provides simple TODO tracking functionality

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Blocked,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

/// A single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: u64,
    pub updated_at: u64,
    pub completed_at: Option<u64>,
    pub assignee: Option<String>,
    pub tags: Vec<String>,
    pub subtasks: Vec<String>, // Subtask IDs
    pub dependencies: Vec<String>, // Task IDs this depends on
}

impl Task {
    pub fn new(id: String, title: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Task {
            id,
            title,
            description: String::new(),
            status: TaskStatus::Pending,
            priority: TaskPriority::default(),
            created_at: now,
            updated_at: now,
            completed_at: None,
            assignee: None,
            tags: Vec::new(),
            subtasks: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn set_completed(&mut self) {
        self.status = TaskStatus::Completed;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.completed_at = Some(self.updated_at);
    }
}

/// Project containing multiple tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub tasks: HashMap<String, Task>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Project {
            name,
            description: String::new(),
            tasks: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    pub fn remove_task(&mut self, id: &str) -> Option<Task> {
        self.tasks.remove(id)
    }

    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(id)
    }

    pub fn tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks.values()
            .filter(|t| t.status == status)
            .collect()
    }

    pub fn tasks_by_priority(&self, priority: TaskPriority) -> Vec<&Task> {
        self.tasks.values()
            .filter(|t| t.priority == priority)
            .collect()
    }

    pub fn completion_percentage(&self) -> f64 {
        if self.tasks.is_empty() {
            return 100.0;
        }
        let completed = self.tasks.values()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        (completed as f64 / self.tasks.len() as f64) * 100.0
    }
}

/// Task tracker manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTracker {
    pub projects: HashMap<String, Project>,
    pub active_project: Option<String>,
}

impl Default for TaskTracker {
    fn default() -> Self {
        TaskTracker {
            projects: HashMap::new(),
            active_project: None,
        }
    }
}

impl TaskTracker {
    pub fn new() -> Self {
        TaskTracker::default()
    }

    pub fn create_project(&mut self, name: String) -> &mut Project {
        self.projects.insert(name.clone(), Project::new(name));
        self.projects.get_mut(&self.projects.keys().last().unwrap().clone()).unwrap()
    }

    pub fn get_active_project(&mut self) -> Option<&mut Project> {
        self.active_project.as_ref().and_then(|name| self.projects.get_mut(name))
    }

    pub fn set_active_project(&mut self, name: &str) -> bool {
        if self.projects.contains_key(name) {
            self.active_project = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn add_task_to_active(&mut self, task: Task) -> bool {
        if let Some(project) = self.get_active_project() {
            project.add_task(task);
            true
        } else {
            false
        }
    }
}

/// Simple task list for quick TODO tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: usize,
    pub text: String,
    pub completed: bool,
    pub created_at: u64,
}

impl TodoList {
    pub fn new() -> Self {
        TodoList {
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, text: String) -> usize {
        let id = self.items.len() + 1;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.items.push(TodoItem {
            id,
            text,
            completed: false,
            created_at: now,
        });

        id
    }

    pub fn toggle(&mut self, id: usize) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.completed = !item.completed;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, id: usize) -> bool {
        let initial_len = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() < initial_len
    }

    pub fn clear_completed(&mut self) {
        self.items.retain(|i| !i.completed);
    }

    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|i| i.completed).count()
    }

    pub fn pending_count(&self) -> usize {
        self.items.iter().filter(|i| !i.completed).count()
    }
}

impl Default for TodoList {
    fn default() -> Self {
        TodoList::new()
    }
}
