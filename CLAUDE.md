# jav-rs

Rust TUI client for JavBus. Search movies, browse magnets, manage history/favorites — all in terminal with image preview support (Kitty protocol).

## Tech Stack

- **Language**: Rust 2024 edition
- **TUI**: ratatui 0.29 + crossterm 0.28
- **Image**: ratatui-image 4 (Kitty/Sixel protocol) + image 0.25
- **Async**: tokio (full features)
- **HTTP**: reqwest 0.12 (with cookies)
- **Scraping**: scraper 0.22 (CSS selectors on HTML)
- **Storage**: rusqlite 0.32 (bundled SQLite)
- **CLI**: clap 4 (derive)
- **Error**: anyhow 1

## Project Structure

```
src/
├── main.rs              # Entry point: CLI args (--base, --proxy), init client/store/tui
├── scraper/
│   ├── mod.rs           # Re-exports JavClient, Movie, Magnet, PageResult
│   ├── types.rs         # Data types: Movie, Magnet, PageResult
│   └── client.rs        # JavClient: HTTP fetch + HTML parse (pages, magnets, images)
├── tui/
│   ├── mod.rs           # Re-exports run()
│   └── app.rs           # App state, event loop, all UI rendering (670 lines)
└── store/
    └── mod.rs           # SQLite Store: history CRUD, favorites toggle
```

## Architecture

- **Async message passing**: App uses `mpsc::UnboundedSender/Receiver<AsyncMsg>` for non-blocking I/O
- **Event loop**: 50ms poll with crossterm events + async message processing each frame
- **Dual panel**: Left (movie list with thumbnails) / Right (cover image + magnet list)
- **Popup search**: Modal search dialog overlays main UI
- **Image rendering**: Kitty protocol via ratatui-image; thumbnails cached in HashMap

## Build & Run

```bash
cargo build --release
cargo run -- --base https://www.javbus.com --proxy socks5://127.0.0.1:1080
```

## Key Bindings

| Key | Action |
|-----|--------|
| `f` or `/` | Open search popup |
| `j/k` or arrows | Navigate list / magnets |
| `Enter` | View movie details + magnets |
| `Tab` | Switch panel (left/right) |
| `n/p` | Next/prev page |
| `Esc` | Back to left panel |
| `q` | Quit |

## Data

SQLite database at `~/.jav/data.db` with `history` table (number, title, link, magnets, favorited).

## Conventions

- No `unwrap()` on fallible operations in production paths; use `anyhow::Result`
- Selectors are constructed per-call (could be optimized to lazy_static if needed)
- Image fetching is fire-and-forget via tokio::spawn
- All user-facing strings are in Chinese
