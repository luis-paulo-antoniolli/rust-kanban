# Kanban CLI

A powerful, hierarchical Kanban board for your terminal, written in Rust.

![Kanban CLI Demo](https://via.placeholder.com/800x400?text=Kanban+CLI+Screenshot)

## Features

- ** hierarchical Structure**: Create boards within boards. Organize your projects with infinite depth.
- **Multiple Content Types**:
    - **Boards**: Standard Kanban columns (To Do, In Progress, Done).
    - **Todo Lists**: Simple checkbox lists for smaller tasks.
    - **Text Notes**: Rich text editor for detailed notes or documentation.
- **Vim-like Navigation**: Navigate your boards with `h`, `j`, `k`, `l` keys.
- **Instant Persistence**: Your data is saved automatically and efficiently using `sled` embedded database.
- **Fast & Lightweight**: Built with Rust and Ratatui for blazing fast performance.

## Installation

Ensure you have Rust installed.

```bash
git clone https://github.com/yourusername/kanban-cli.git
cd kanban-cli
cargo run --release
```

## Keybindings

### Global
- `q`: Quit application
- `?`: Toggle Help

### Navigation
- `h` / `Left`: Move cursor left
- `j` / `Down`: Move cursor down
- `k` / `Up`: Move cursor up
- `l` / `Right`: Move cursor right
- `Enter`: Drill down into a card (open board/todo/note)
- `Esc`: Go back to parent board

### Board Actions
- `Shift` + `h` / `ArrowLeft`: Move selected task to the left column
- `Shift` + `l` / `ArrowRight`: Move selected task to the right column
- `a`: Add new item to current column
- `d`: Delete selected item

### Todo List Actions
- `Space`: Toggle checkbox
- `a`: Add new item
- `d`: Delete item

### Text Note Actions
- `Enter`: Start editing text
- `Esc`: Stop editing

## Technology Stack

- **Rust**: Core language
- **Ratatui**: Terminal User Interface library
- **Crossterm**: Terminal manipulation
- **Sled**: Embedded database for persistence
- **Serde**: Serialization/Deserialization

## License

MIT
