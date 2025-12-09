use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Board {
    pub title: String,
    pub columns: Vec<Column>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            title: "Main Board".to_string(),
            columns: vec![
                Column::new("To Do"),
                Column::new("In Progress"),
                Column::new("Done"),
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Column {
    pub title: String,
    pub tasks: Vec<Task>,
}

impl Column {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            tasks: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub content: Option<TaskContent>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskContent {
    Board(Board),
    Todo(Vec<TodoItem>),
    Text(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TodoItem {
    pub text: String,
    pub done: bool,
}

impl Task {
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            content: None,
        }
    }


}
