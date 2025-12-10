use std::io;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod model;
mod ui;

use app::{App, Action, InputMode};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?; 
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('?') && app.input_mode != InputMode::Editing {
                    app.update(Action::ToggleHelp)?;
                    continue;
                }
                
                if app.show_help {
                     app.update(Action::ToggleHelp)?;
                     continue;
                }

                let action = match app.input_mode {
                    InputMode::Normal => {
                        // Check for Shift modifier FIRST
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            match key.code {
                                KeyCode::Left | KeyCode::Char('H') => Some(Action::MoveTaskLeft),
                                KeyCode::Right | KeyCode::Char('L') => Some(Action::MoveTaskRight),
                                _ => None,
                            }
                        } else {
                            match key.code {
                                KeyCode::Char('q') => Some(Action::Quit),
                                KeyCode::Left | KeyCode::Char('h') => Some(Action::MoveLeft),
                                KeyCode::Right | KeyCode::Char('l') => Some(Action::MoveRight),
                                KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
                                KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
                                KeyCode::Enter => Some(Action::DrillDown),
                                KeyCode::Backspace | KeyCode::Esc => Some(Action::GoBack),
                                KeyCode::Char('a') => Some(Action::EnterEditMode),
                                KeyCode::Char('c') => Some(Action::EnterAddColumnMode),
                                KeyCode::Char('d') => Some(Action::DeleteTask),
                                KeyCode::Char(' ') => Some(Action::ToggleTodo),
                                
                                // Alternative shift bindings if terminal swallows modifiers for arrows (sometimes tricky)
                                KeyCode::Char('H') => Some(Action::MoveTaskLeft), // Shift+h
                                KeyCode::Char('L') => Some(Action::MoveTaskRight), // Shift+l
                                _ => None,
                            }
                        }
                    },
                    InputMode::Editing | InputMode::EditingColumn => match key.code {
                        KeyCode::Enter => Some(Action::SubmitTask),
                        KeyCode::Esc => Some(Action::ExitEditMode),
                        KeyCode::Char(c) => Some(Action::InputChar(c)),
                        KeyCode::Backspace => Some(Action::InputBackspace),
                        _ => None,
                    },
                    InputMode::SelectType => match key.code {
                        KeyCode::Char('b') => Some(Action::SelectBoard),
                        KeyCode::Char('t') => Some(Action::SelectTodo),
                        KeyCode::Char('n') => Some(Action::SelectText),
                        KeyCode::Esc => Some(Action::GoBack),
                        _ => None,
                    },
                };

                if let Some(action) = action {
                    app.update(action)?;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
