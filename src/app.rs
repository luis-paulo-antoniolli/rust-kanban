use crate::model::{Board, Task, TaskContent, TodoItem};
use anyhow::Result;
use bincode::config;
use std::fs;
use std::path::PathBuf;

const DB_FILE: &str = "kanban.db";

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    EditingColumn, // New mode for adding columns
    SelectType, // New mode for choosing content type
}

#[derive(Debug, Clone)]
pub enum Action {
    Quit,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveTaskLeft,
    MoveTaskRight,
    DrillDown,
    GoBack,
    EnterEditMode,
    EnterAddColumnMode, // New action
    ExitEditMode,
    InputChar(char),
    InputBackspace,
    SubmitTask,
    DeleteTask,
    ToggleTodo, // New
    ToggleHelp, // New
    SelectBoard,
    SelectTodo,
    SelectText,
}

pub struct App {
    pub root: Board,
    pub path: Vec<(usize, usize)>, // Path to current context (col_idx, task_idx)
    pub cursor: (usize, usize),    // (col, row) or (item_idx, 0) for lists
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub should_quit: bool,
    pub show_help: bool,
    pub dirty: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        // Simple file path
        let path = PathBuf::from(DB_FILE);
        
        let root = if path.exists() {
            let data = fs::read(&path)?;
            // Try Bincode
            if let Ok(board) = bincode::serde::decode_from_slice(&data, config::standard()).map(|(b, _)| b) {
                board
            } else {
                 Board::default()
            }
        } else {
             Board::default()
        };

        Ok(Self {
            root,
            path: Vec::new(),
            cursor: (0, 0),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            should_quit: false,
            show_help: false,
            dirty: false,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        let bytes = bincode::serde::encode_to_vec(&self.root, config::standard())?;
        fs::write(DB_FILE, bytes)?;
        self.dirty = false;
        Ok(())
    }

    pub fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.should_quit = true,

            Action::ToggleHelp => self.show_help = !self.show_help,
            
            // Navigation
            Action::MoveUp => self.move_cursor(0, -1),
            Action::MoveDown => self.move_cursor(0, 1),
            Action::MoveLeft => self.move_cursor(-1, 0),
            Action::MoveRight => self.move_cursor(1, 0),
            Action::MoveTaskLeft => self.move_task_horizontal(-1),
            Action::MoveTaskRight => self.move_task_horizontal(1),
            
            Action::DrillDown => self.handle_drill_down(),
            Action::GoBack => self.go_back(),
            
            // Editing
            Action::EnterEditMode => {
                if !self.show_help {
                     // Check if valid context for adding tasks (Board or Todo)
                     // Using short block to limit borrow scope
                     let can_edit = matches!(self.get_active_content(), ActiveContentRef::Board(_) | ActiveContentRef::Todo(_));
                     if can_edit {
                        self.input_mode = InputMode::Editing;
                     } 
                }
            },
            Action::EnterAddColumnMode => {
                if !self.show_help {
                    // Only allow adding columns if we are viewing a board
                    if let ActiveContentRef::Board(_) = self.get_active_content() {
                        self.input_mode = InputMode::EditingColumn;
                    }
                }
            },
            Action::ExitEditMode => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            Action::InputChar(c) => self.input_buffer.push(c),
            Action::InputBackspace => { self.input_buffer.pop(); },
            Action::SubmitTask => self.submit_input(),
            
            Action::DeleteTask => self.delete_item(),
            Action::ToggleTodo => self.toggle_todo(),
            
            // Type Selection
            Action::SelectBoard => self.initialize_content(TaskContent::Board(Board { title: "New Board".into(), ..Default::default() })),
            Action::SelectTodo => self.initialize_content(TaskContent::Todo(Vec::new())),
            Action::SelectText => self.initialize_content(TaskContent::Text(String::new())),
        }



        // Auto-save only if dirty
        if self.dirty {
            let _ = self.save();
        }

        Ok(())
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        if self.input_mode != InputMode::Normal || self.show_help { return; }

        match self.get_active_content() {
            ActiveContentRef::Board(board) => {
                let col_count = board.columns.len();
                if col_count == 0 { return; }
                let (mut c, mut r) = (self.cursor.0 as i32, self.cursor.1 as i32);
                
                // Horizontal
                if dx != 0 { c = (c + dx).clamp(0, col_count as i32 - 1); }
                
                // Vertical
                let tasks_len = board.columns[c as usize].tasks.len();
                let max_r = if tasks_len > 0 { tasks_len as i32 - 1 } else { 0 };
                
                if dy != 0 {
                    if dx != 0 { r = r.min(max_r); } // moved col, clamp row
                    else { r = (r + dy).clamp(0, max_r); }
                } else if dx != 0 {
                    r = r.min(max_r);
                }

                self.cursor = (c as usize, r as usize);
            },
            ActiveContentRef::Todo(items) => {
                let len = items.len();
                if len == 0 { return; }
                let mut r = self.cursor.1 as i32;
                if dy != 0 { r = (r + dy).clamp(0, len as i32 - 1); }
                self.cursor = (0, r as usize);
            },
            ActiveContentRef::Text(_) => {
                // No cursor movement in text view for now (view only)
            },
            ActiveContentRef::None => {},
        }
    }

    fn handle_drill_down(&mut self) {
        if let ActiveContentRef::Board(board) = self.get_active_content() {
            let (c, r) = self.cursor;
            if let Some(col) = board.columns.get(c) {
                if let Some(task) = col.tasks.get(r) {
                    if task.content.is_none() {
                        self.input_mode = InputMode::SelectType;
                    } else {
                        // Push path
                        self.path.push((c, r));
                        self.cursor = (0, 0);
                        
                        // If it's text, auto-enter edit mode? 
                        // Let's keep it view-only first, then Enter again to edit?
                        // For simplicity: If entering Text content, we just view it. 
                        // User can press 'Enter' inside Text view to edit (implemented below).
                        if let ActiveContentRef::Text(text) = self.get_active_content() {
                             let text_content = text.clone();
                             self.input_mode = InputMode::Editing;
                             self.input_buffer = text_content;
                        }
                    }
                }
            }
        } else if let ActiveContentRef::Text(_) = self.get_active_content() {
            // If already in text view, Enter to edit
             if let ActiveContentRef::Text(text) = self.get_active_content() {
                 let text_content = text.clone();
                 self.input_mode = InputMode::Editing;
                 self.input_buffer = text_content;
             }
        }
    }

    fn go_back(&mut self) {
        if self.show_help {
            self.show_help = false;
            return;
        }
        if self.input_mode == InputMode::SelectType {
            self.input_mode = InputMode::Normal;
            return;
        }
        if let Some((col, row)) = self.path.pop() {
            self.cursor = (col, row);
        }
    }

    fn initialize_content(&mut self, content: TaskContent) {
         if self.input_mode != InputMode::SelectType { return; }
         
         // We need to set the content of the *current* selection (which is the parent's cursor)
         // Wait, we are in SelectType mode, meaning we haven't pushed to path yet.
         // We are sitting at the parent board.
         
         // Helper to mutate current selection
         {
         let (c, r) = self.cursor;
         // We need to get the PARENT board.
         let board = Self::get_board_recursive(&mut self.root, &self.path); // This gets the board we are LOOKING at.
         if let Some(col) = board.columns.get_mut(c) {
             if let Some(task) = col.tasks.get_mut(r) {
                 task.content = Some(content.clone());
                 self.dirty = true;
             }
         }
         }
         
         self.input_mode = InputMode::Normal;
         // Automatically drill down after creation
         self.handle_drill_down();
    }

    fn submit_input(&mut self) {
        if self.input_mode == InputMode::EditingColumn {
            let title = self.input_buffer.trim().to_string();
            if !title.is_empty() {
                 let board = Self::get_board_recursive(&mut self.root, &self.path);
                 board.columns.push(crate::model::Column::new(&title));
                 self.dirty = true;
            }
            self.input_buffer.clear();
            self.input_mode = InputMode::Normal;
            return;
        }

        match self.get_active_content() {
            ActiveContentRef::Board(_) => {
                // Adding variable to avoid borrow checker hell
                let title = self.input_buffer.trim().to_string();
                if !title.is_empty() {
                    let (c, _) = self.cursor;
                    let board = Self::get_board_recursive(&mut self.root, &self.path);
                    if c < board.columns.len() {
                        board.columns[c].tasks.push(Task::new(&title, ""));
                        self.dirty = true;
                    }
                }
            },
            ActiveContentRef::Todo(_) => {
                let text = self.input_buffer.trim().to_string();
                if !text.is_empty() {
                    let _items: Vec<TodoItem> = Vec::new(); // placeholder - unused logic branch if we just want to submit simple task?
                    // Actually, for Todo list, we add items via `a`. SubmitTask is for `Editing` mode.
                    // If we are in `Editing` mode inside a Todo view?
                    // We only use Editing mode for *renaming*? Or adding?
                    // My logic in `submit_input`: `match active_content`.
                    // If `ActiveContent::Todo`, `input_buffer` is the new item text?
                    // Yes.
                    
                    let text = self.input_buffer.trim().to_string();
                    if !text.is_empty() {
                         self.add_todo_item(text);
                         // self.dirty set inside add_todo_item
                    }
                }
            },
            ActiveContentRef::Text(_) => {
                // Saving text content
                let text = self.input_buffer.clone();
                self.set_text_content(text);
                // self.dirty set inside set_text_content
            },
             _ => {}
        }
        self.input_buffer.clear();
        self.input_mode = InputMode::Normal;
    }

    fn delete_item(&mut self) {
        match self.get_active_content() {
            ActiveContentRef::Board(board) => {
                let (c, r) = self.cursor;
                if c < board.columns.len() && r < board.columns[c].tasks.len() {
                    let board_mut = Self::get_board_recursive(&mut self.root, &self.path);
                    board_mut.columns[c].tasks.remove(r);
                    self.dirty = true;
                    // Adjust cursor
                     if r >= board_mut.columns[c].tasks.len() && r > 0 {
                        self.cursor.1 -= 1;
                    }
                }
            },
            ActiveContentRef::Todo(items) => {
                let r = self.cursor.1;
                if r < items.len() {
                   self.remove_todo_item(r);
                   // self.dirty handled inside
                   if r > 0 { self.cursor.1 = r.saturating_sub(1); }
                }
            },
            _ => {}
        }
    }

    fn toggle_todo(&mut self) {
        if let ActiveContentRef::Todo(items) = self.get_active_content() {
            let r = self.cursor.1;
            if r < items.len() {
                self.toggle_todo_item(r);
            }
        }
    }

    // --- Helpers / View Logic ---

    pub fn get_breadcrumbs(&self) -> Vec<String> {
        let mut crumbs = vec!["Main Board".to_string()];
        let mut board = &self.root;
        
        for &(col_idx, task_idx) in &self.path {
            if let Some(col) = board.columns.get(col_idx) {
                if let Some(task) = col.tasks.get(task_idx) {
                    crumbs.push(task.title.clone());
                    if let Some(TaskContent::Board(ref b)) = task.content {
                        board = b;
                    } 
                }
            }
        }
        crumbs
    }

    pub fn get_active_content(&self) -> ActiveContentRef<'_> {
        // Traverse to the tip of path
        let mut board = &self.root;

        for &(col_idx, task_idx) in &self.path {
            if let Some(col) = board.columns.get(col_idx) {
                if let Some(task) = col.tasks.get(task_idx) {
                    if let Some(TaskContent::Board(ref b)) = task.content {
                        board = b;
                    } else {
                        // Leaf is not a board, so return its content
                        if let Some(ref content) = task.content {
                            match content {
                                TaskContent::Todo(items) => return ActiveContentRef::Todo(items),
                                TaskContent::Text(txt) => return ActiveContentRef::Text(txt),
                                TaskContent::Board(_) => {}
                            }
                        } else {
                             return ActiveContentRef::None;
                        }
                    }
                }
            }
        }
        ActiveContentRef::Board(board)
    }



    fn get_board_recursive<'a>(board: &'a mut Board, path: &[(usize, usize)]) -> &'a mut Board {
        if path.is_empty() {
             return board;
        }
        let (col_idx, task_idx) = path[0];
        // We assume valid path
        if let Some(TaskContent::Board(ref mut b)) = board.columns[col_idx].tasks[task_idx].content {
            return Self::get_board_recursive(b, &path[1..]);
        }
        
        // If we are here, logic error (asking for board but found something else)
        panic!("Invalid path: expected Board");
    }

    fn add_todo_item(&mut self, text: String) {
        // We want the task at `self.path`.
        if let Some(task) = Self::get_task_mut_recursive(&mut self.root, &self.path) {
            if let Some(TaskContent::Todo(ref mut items)) = task.content {
                items.push(TodoItem { text, done: false });
                items.sort_by_key(|k| k.done);
                self.dirty = true;
            }
        }
    }

    fn remove_todo_item(&mut self, index: usize) {
        if let Some(task) = Self::get_task_mut_recursive(&mut self.root, &self.path) {
            if let Some(TaskContent::Todo(ref mut items)) = task.content {
                if index < items.len() { 
                    items.remove(index); 
                    self.dirty = true;
                }
            }
        }
    }

    fn toggle_todo_item(&mut self, index: usize) {
        if let Some(task) = Self::get_task_mut_recursive(&mut self.root, &self.path) {
             if let Some(TaskContent::Todo(ref mut items)) = task.content {
                 if let Some(item) = items.get_mut(index) {
                     item.done = !item.done;
                     self.dirty = true;
                 }
                 items.sort_by_key(|k| k.done);
             }
        }
    }

    fn set_text_content(&mut self, text: String) {
        if let Some(task) = Self::get_task_mut_recursive(&mut self.root, &self.path) {
            task.content = Some(TaskContent::Text(text));
            self.dirty = true;
        }
    }

    fn move_task_horizontal(&mut self, dir: i32) {
        if self.input_mode != InputMode::Normal { return; }
        
        // Only works if active content is a Board (tasks move between columns)
        if let ActiveContentRef::Board(board) = self.get_active_content() {
             let (c, r) = self.cursor;
             let new_c = c as i32 + dir;
             
             // Check bounds
             if new_c < 0 || new_c >= board.columns.len() as i32 {
                 return;
             }
             let new_c = new_c as usize;
             
              // Mutate
              {
                  let board_mut = Self::get_board_recursive(&mut self.root, &self.path);
                  if r < board_mut.columns[c].tasks.len() {
                     let task = board_mut.columns[c].tasks.remove(r);
                     board_mut.columns[new_c].tasks.push(task);
                     self.dirty = true;
                     
                     // Adjust cursor
                     // If we moved right, we are now at the bottom of new_c? 
                     // Or should we try to stay at same relative index?
                     // Standard Kanban: Move to bottom of new column usually.
                     // But let's just update cursor to follow the task at the end of new list
                     
                     self.cursor = (new_c, board_mut.columns[new_c].tasks.len() - 1);
                     
                     // Also need to clamp the OLD column cursor if we were not at the bottom?
                     // Actually, since we switch `self.cursor.0` to `new_c`, we don't care about old column row index anymore,
                     // except if we move BACK? 
                     // Wait, `cursor` is `(col, row)`.
                     // If we just changed columns, we are fine.
                 }
             }
        }
    }

    fn get_task_mut_recursive<'a>(board: &'a mut Board, path: &[(usize, usize)]) -> Option<&'a mut Task> {
        if path.is_empty() { return None; }
        let (col_idx, task_idx) = path[0];
        
        if path.len() == 1 {
            return board.columns.get_mut(col_idx).and_then(|c| c.tasks.get_mut(task_idx));
        }

        let task = &mut board.columns[col_idx].tasks[task_idx];
        if let Some(TaskContent::Board(ref mut sub)) = task.content {
            return Self::get_task_mut_recursive(sub, &path[1..]);
        }
        None
    }
}

// Helper enum to avoid cloning huge boards constantly? 
// Actually we clone board for `get_active_content` which is not ideal for performance but fine for CLI.
// Optimization: Return Cow or references? Complex with App struct borrowing.
// For now, cloning Board is okay-ish if deep trees aren't huge.
pub enum ActiveContentRef<'a> {
    Board(&'a Board),
    Todo(&'a Vec<TodoItem>),
    Text(&'a String),
    None,
}
