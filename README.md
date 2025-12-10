# ⚡ Kanban CLI

A **ultra-high performance**, hierarchical Kanban board for your terminal, written in Rust.

Designed to run efficiently on **any hardware**, from a modern Threadripper to a legacy Pentium 4.

![Kanban CLI Demo](https://via.placeholder.com/800x400?text=Kanban+CLI+Screenshot)

## 🚀 Why is it so fast? (Architecture)

This project was built with a generic "Performance First" philosophy. Every architectural choice prioritizes speed and low resource usage:

*   **Zero-Cost Abstractions (Rust)**: No Garbage Collector pauses, deterministic memory usage.
*   **0% CPU Idle**: The event loop is blocking. If you aren't typing, the application uses literally **0% CPU**.
*   **Binary Database**: Instead of slow JSON/XML parsing, we use **Bincode**. It's smaller, faster, and requires almost no CPU to serialize/deserialize.
*   **Zero-Copy Design**: The internal architecture uses `ActiveContentRef` (borrowed types) to display data without cloning heavy structures in memory.
*   **Smart Rendering**: Uses `Ratatui`'s buffer diffing. Only screen cells that actually changed are redrawn.

## 🛠️ Build & Install (Max Performance)

This project is configured to automatically detect your CPU (Native Compilation) and optimize the code specifically for your machine's instruction set (AVX, SSE, etc).

### Standard Build (Recommended)
This commands enables all optimizations (`lto`, `strip`, `opt-level=3`) but takes longer to compile.

```bash
git clone https://github.com/SEU_USUARIO/kanban-cli
cd kanban-cli
# "Release" enables compiler optimizations. 
# Without this flag, it runs in "debug" mode (slow).
cargo build --release
```

After building, the optimized binary will be in `./target/release/kanban-cli`.

### Compatibility
*   **Supported OS**: Linux, Windows (inc. 32-bit), macOS, FreeBSD.
*   **Requirements**: Rust toolchain.

> **Note**: For 32-bit systems (Pentium 4 era), use `rustup target add i686-pc-windows-msvc` before building.

## ✨ Features

- **Hierarchical Structure**: Boards within boards within boards.
- **Vim-like Navigation**: `h`, `j`, `k`, `l` for speed.
- **Multiple Content Types**: Boards, Todo Lists, and Text Notes.
- **Instant Startup**: Sub-millisecond launch time.

## ⌨️ keybindings

### Global
- `q`: Quit
- `?`: Toggle Help

### Navigation
- `h` / `Left`: Move cursor left
- `j` / `Down`: Move cursor down
- `k` / `Up`: Move cursor up
- `l` / `Right`: Move cursor right
- `Enter`: Open card
- `Esc`: Go back

### Editing
- `a`: Add new item
- `d`: Delete item
- `Space`: Toggle Todo check
- `Shift` + `H/L`: Move tasks (Kanban)

## License
MIT
