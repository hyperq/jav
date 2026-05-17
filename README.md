# 🎬 JAV TUI

<p align="center">
  <strong>A beautiful terminal client for JavBus with image preview support</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-brightgreen?style=flat-square" />
  <img src="https://img.shields.io/badge/theme-Catppuccin%20Frappé-lavender?style=flat-square" />
</p>

<p align="center">
  <a href="README_CN.md">🇨🇳 中文文档</a>
</p>

---

## ✨ Features

| | Feature | Description |
|---|---------|-------------|
| 🔍 | **Movie Search** | Search by keyword, number, censored/uncensored |
| 👩 | **Actress Browser** | Grid view with avatars, search by name or code |
| 🖼️ | **Image Preview** | Native terminal images via Kitty graphics protocol |
| 🧲 | **Smart Magnets** | Auto-sort by subtitle > HD > size, one-key grab |
| ✅ | **Multi-select** | Select across searches, batch export magnets |
| 📋 | **History & Favorites** | Persistent with SQLite, cover image cache |
| ☁️ | **115 Cloud** | QR login, push magnets to 115 offline download |
| ⚙️ | **Configurable** | Toggle images, sort preferences, all in SQLite |
| 🎨 | **Catppuccin Frappé** | Beautiful, eye-friendly color theme |

## 🚀 Install

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/hyperq/jav/master/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/hyperq/jav/master/install.ps1 | iex
```

**From source:**

```bash
git clone https://github.com/hyperq/jav.git
cd jav
cargo build --release
cp target/release/jav ~/.local/bin/
```

## 📖 Usage

```bash
# Basic
jav

# With proxy
jav --proxy socks5://127.0.0.1:1080

# Custom site
jav --base https://www.javbus.com
```

## ⌨️ Keybindings

### 🔍 Search

| Key | Action |
|-----|--------|
| `f` | Search movies |
| `F` | Search actresses |
| `S` | Actress code direct (e.g. `okq`) |
| `N` | Jump to page number |

### 🧭 Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Up / Down |
| `h` / `l` | Left / Right (grid columns) |
| `Enter` | Open detail |
| `Tab` | Switch panel |
| `n` | Load more |
| `Scroll` | Mouse scroll |
| `Click` | Click to select |

### 📦 Actions

| Key | Action |
|-----|--------|
| `Space` | Toggle select |
| `a` | Select all |
| `e` | Export magnets to file |
| `s` | Toggle favorite |
| `g` | Grab best magnet |

### ☁️ 115 Cloud

| Key | Action |
|-----|--------|
| `d` | Download current to 115 |
| `D` | Batch download selected |
| `L` | QR code login |

### 🛠️ Other

| Key | Action |
|-----|--------|
| `c` | Settings |
| `~` | Toggle log panel |
| `q` | Quit |

## 📁 Data

All data stored in `~/.jav/`:

```
~/.jav/
├── data.db          # SQLite: history, favorites, config, magnets
├── cache/           # Image cache (thumbnails, covers, avatars)
└── 115_cookie.json  # 115 cloud login session
```

## 🖥️ Terminal Support

| Terminal | Images | Status |
|----------|--------|--------|
| Ghostty | ✅ Kitty protocol | Recommended |
| Kitty | ✅ Native | Full support |
| WezTerm | ✅ Kitty protocol | Full support |
| iTerm2 | ❌ | Text-only mode |
| Alacritty | ❌ | Text-only mode |
| Terminal.app | ❌ | Text-only mode |

> Unsupported terminals automatically fall back to text-only mode.

## 📄 License

MIT © [hyperq](https://github.com/hyperq)
