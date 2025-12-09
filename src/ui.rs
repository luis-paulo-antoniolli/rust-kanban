use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Clear, Wrap, Table, Row},
    Frame,
};
use crate::app::{App, InputMode, ActiveContent};
use crate::model::TaskContent;

// Theme Constants

const COLOR_BORDER_ACTIVE: Color = Color::Green;
const COLOR_BORDER_INACTIVE: Color = Color::DarkGray;
const COLOR_SELECTED_BG: Color = Color::Blue;
const COLOR_SELECTED_FG: Color = Color::White;
const COLOR_BOARD_ICON: Color = Color::Yellow;
const COLOR_TODO_ICON: Color = Color::Cyan;
const COLOR_TEXT_ICON: Color = Color::Magenta;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Footer / Help
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    
    // Determine what to draw based on active content
    match app.get_active_content() {
        ActiveContent::Board(board) => draw_board(f, app, &board, chunks[1]),
        ActiveContent::Todo(items) => draw_todo(f, app, &items, chunks[1]),
        ActiveContent::Text(text) => draw_text_view(f, app, &text, chunks[1]),
        ActiveContent::None => draw_empty_selection(f, chunks[1]), 
    }

    draw_footer(f, app, chunks[2]);

    if app.input_mode == InputMode::Editing {
        draw_input_popup(f, app);
    } else if app.input_mode == InputMode::SelectType {
        draw_type_selection_popup(f);
    }
    
    if app.show_help {
        draw_help_popup(f);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let raw_crumbs = app.get_breadcrumbs();
    let mut spans = Vec::new();
    
    for (i, crumb) in raw_crumbs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" > ", Style::default().fg(Color::DarkGray)));
        }
        if i == raw_crumbs.len() - 1 {
            // Active
             spans.push(Span::styled(crumb, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        } else {
             spans.push(Span::raw(crumb));
        }
    }

    let title = Paragraph::new(Line::from(spans))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_BORDER_INACTIVE))
            .title(" Kanban CLI ")
            .title_alignment(Alignment::Center));
    
    f.render_widget(title, area);
}

fn draw_board(f: &mut Frame, app: &App, board: &crate::model::Board, area: Rect) {
    let col_count = board.columns.len();

    if col_count == 0 {
        let text = Paragraph::new("No columns defined.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(text, area);
        return;
    }

    let constraints: Vec<Constraint> = (0..col_count)
        .map(|_| Constraint::Percentage(100 / col_count as u16))
        .collect();
    
    let col_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, column) in board.columns.iter().enumerate() {
        let is_selected_col = i == app.cursor.0;
        
        let items: Vec<ListItem> = column.tasks.iter().enumerate().map(|(j, task)| {
            let is_selected_task = is_selected_col && j == app.cursor.1;
            
            let (bg, fg) = if is_selected_task {
                (COLOR_SELECTED_BG, COLOR_SELECTED_FG)
            } else {
                (Color::Reset, Color::White)
            };

            let (marker, marker_color) = match &task.content {
                Some(TaskContent::Board(_)) => ("📂 ", COLOR_BOARD_ICON),
                Some(TaskContent::Todo(_)) => ("☑️ ", COLOR_TODO_ICON),
                Some(TaskContent::Text(_)) => ("📝 ", COLOR_TEXT_ICON),
                None => ("📄 ", Color::DarkGray),
            };

            let content = Line::from(vec![
                Span::styled(marker, Style::default().fg(marker_color)),
                Span::raw(&task.title),
            ]);
            
            ListItem::new(content)
                .style(Style::default().bg(bg).fg(fg))
        }).collect();

        let border_style = if is_selected_col {
            Style::default().fg(COLOR_BORDER_ACTIVE).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(COLOR_BORDER_INACTIVE)
        };
        
        // Add bold to column title if active
        let title_style = if is_selected_col {
             Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
             Style::default().fg(Color::White)
        };

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(format!(" {} ({}) ", column.title, column.tasks.len()), title_style))
                .border_style(border_style));
        
        f.render_widget(list, col_chunks[i]);
    }
}

fn draw_todo(f: &mut Frame, app: &App, items: &[crate::model::TodoItem], area: Rect) {
    let pending_items: Vec<(usize, &crate::model::TodoItem)> = items.iter().enumerate().filter(|(_, i)| !i.done).collect();
    let done_items: Vec<(usize, &crate::model::TodoItem)> = items.iter().enumerate().filter(|(_, i)| i.done).collect();
    
    let constraints = if pending_items.is_empty() && done_items.is_empty() {
        vec![Constraint::Percentage(100)]
    } else if pending_items.is_empty() {
         vec![Constraint::Percentage(0), Constraint::Percentage(100)]
    } else if done_items.is_empty() {
         vec![Constraint::Percentage(100), Constraint::Percentage(0)]
    } else {
         vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
        
    // Pending List
    if !pending_items.is_empty() || done_items.is_empty() {
        let list_items: Vec<ListItem> = pending_items.iter().map(|&(i, item)| {
             let is_selected = i == app.cursor.1;
             let style = if is_selected {
                 Style::default().fg(COLOR_SELECTED_FG).bg(COLOR_SELECTED_BG)
             } else {
                 Style::default()
             };
             ListItem::new(format!("[ ] {}", item.text)).style(style)
        }).collect();
        
        // Ensure we show title even if empty only if it's the only view? 
        // No, show "Pending"
        let list = List::new(list_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" To Do ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_BORDER_ACTIVE)));
        f.render_widget(list, chunks[0]);
    }

    // Done List
    if !done_items.is_empty() {
        // If pending was empty, chunk index might need care? 
        // With constraints above: 
        // Case 1 (Both): [0] is Pending, [1] is Done.
        // Case 2 (Only Pending): [0] is Pending, [1] is size 0.
        // Case 3 (Only Done): [0] size 0, [1] is Done.
        
        let target_chunk = if pending_items.is_empty() { chunks[1] } else { chunks[1] };
        
        let list_items: Vec<ListItem> = done_items.iter().map(|&(i, item)| {
             let is_selected = i == app.cursor.1;
             let style = if is_selected {
                 Style::default().fg(COLOR_SELECTED_FG).bg(COLOR_SELECTED_BG)
             } else {
                 Style::default().fg(Color::Gray)
             };
             ListItem::new(format!("[x] {}", item.text)).style(style)
        }).collect();
        
        let list = List::new(list_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Done ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(COLOR_BORDER_INACTIVE)));
        f.render_widget(list, target_chunk);
    }
}

fn draw_text_view(f: &mut Frame, _app: &App, text: &str, area: Rect) {
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Notes ")
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_BORDER_ACTIVE)));
    f.render_widget(p, area);
}

fn draw_empty_selection(f: &mut Frame, area: Rect) {
    let p = Paragraph::new("Empty Task. Press Enter to add content.")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(p, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.get_active_content() {
        ActiveContent::Board(_) => "Moves: Shift+Arrows | Enter: Open | a: Add | d: Del | ?: Help",
        ActiveContent::Todo(_) => "Move: jk/Arrows | Space: Toggle | a: Add Item | d: Del | Esc: Back",
        ActiveContent::Text(_) => "Enter: Edit Text | Esc: Back",
        ActiveContent::None => "Enter: Select Content Type | Esc: Back",
    };
    
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, area);
}

fn draw_input_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let title = match app.get_active_content() {
        ActiveContent::Text(_) => " Edit Note ",
        _ => " New Item ",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .title(title)
            .style(Style::default().fg(Color::Blue)));
    
    f.render_widget(input, area);
}

fn draw_type_selection_popup(f: &mut Frame) {
    let area = centered_rect(40, 30, f.area());
    f.render_widget(Clear, area);
    
    let text = vec![
        Line::from("Select Content Type:"),
        Line::from(""),
        Line::from(Span::styled("b - Kanban Board", Style::default().fg(COLOR_BOARD_ICON))),
        Line::from(Span::styled("t - Todo List", Style::default().fg(COLOR_TODO_ICON))),
        Line::from(Span::styled("n - Text Note", Style::default().fg(COLOR_TEXT_ICON))),
    ];
    
    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Create Content "))
        .alignment(Alignment::Center);
    f.render_widget(p, area);
}

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);
    
    let rows = vec![
        Row::new(vec!["Key", "Action"]).style(Style::default().add_modifier(Modifier::BOLD)),
        Row::new(vec!["h / Left", "Move Left"]),
        Row::new(vec!["j / Down", "Move Down"]),
        Row::new(vec!["k / Up", "Move Up"]),
        Row::new(vec!["l / Right", "Move Right"]),
        Row::new(vec!["Shift + ←/→", "Move Task"]),
        Row::new(vec!["Enter", "Drill Down / Edit"]),
        Row::new(vec!["Esc", "Go Back / Cancel"]),
        Row::new(vec!["a", "Add Item"]),
        Row::new(vec!["d", "Delete Item"]),
        Row::new(vec!["Space", "Toggle Todo"]),
        Row::new(vec!["?", "Toggle Help"]),
        Row::new(vec!["q", "Quit"]),
    ];
    
    let table = Table::new(rows, [Constraint::Percentage(30), Constraint::Percentage(70)])
        .block(Block::default().borders(Borders::ALL).title(" Help / Shortcuts ").border_style(Style::default().fg(Color::Yellow)))
        .style(Style::default().fg(Color::White));
        
    f.render_widget(table, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
