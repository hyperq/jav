# JAV TUI

Terminal UI client for JavBus — search movies, browse actresses, manage magnets, with image preview support.

![Rust](https://img.shields.io/badge/rust-2024-orange) ![License](https://img.shields.io/badge/license-MIT-blue)

[中文文档](README_CN.md)

## Features

- **Movie Search** — search by keyword, number, or actress code
- **Actress Browser** — grid view with avatar, search by name
- **Image Preview** — Kitty graphics protocol (Ghostty/Kitty/WezTerm)
- **Magnet Links** — smart sort (caption > HD > size), batch export
- **Multi-select** — space to select, export all magnets to file
- **History & Favorites** — SQLite persistence with cover cache
- **115 Cloud** — QR login, direct download magnets to 115 cloud
- **Configurable** — thumbnails, cover, sort preferences stored in SQLite
- **Catppuccin Frappé** — beautiful color theme

## Install

```bash
cargo build --release
cp target/release/jav ~/.local/bin/
```

## Usage

```bash
# basic
jav

# with proxy
jav --proxy socks5://127.0.0.1:1080

# custom base url
jav --base https://www.javbus.com
```

## Keybindings

| Key | Action |
|-----|--------|
| `f` | Search movies |
| `F` | Search actresses |
| `S` | Actress code direct (e.g. `okq`) |
| `j/k` | Navigate up/down |
| `h/l` | Navigate left/right (grid) |
| `Enter` | Open detail |
| `Tab` | Switch panel (list/detail) |
| `n` | Load more (next page) |
| `N` | Jump to page |
| `Space` | Multi-select toggle |
| `a` | Select all |
| `e` | Export selected magnets |
| `s` | Toggle favorite |
| `g` | Grab best magnet |
| `d` | Download to 115 cloud |
| `D` | Batch download selected to 115 |
| `L` | 115 QR login |
| `c` | Settings |
| `~` | Toggle log panel |
| `q` | Quit |
| Scroll | Mouse scroll support |
| Click | Click list item / tabs |

## Data

All data stored in `~/.jav/`:

| File | Content |
|------|---------|
| `data.db` | SQLite: history, favorites, config, magnets cache |
| `cache/` | Image cache (thumbnails, covers, avatars) |
| `115_cookie.json` | 115 cloud login cookie |

## Requirements

- Rust 1.70+
- Terminal with Kitty graphics protocol (Ghostty, Kitty, WezTerm) for image display
- Falls back to text-only mode on unsupported terminals

## License

MIT
