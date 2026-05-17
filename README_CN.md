# 🎬 JAV TUI

<p align="center">
  <strong>JavBus 终端客户端 — 搜索影片、浏览女优、管理磁力链接，支持终端内图片预览</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-brightgreen?style=flat-square" />
  <img src="https://img.shields.io/badge/theme-Catppuccin%20Frappé-lavender?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">🇺🇸 English</a>
</p>

---

## ✨ 功能

| | 功能 | 说明 |
|---|------|------|
| 🔍 | **影片搜索** | 按关键词、番号搜索，支持有码/无码切换 |
| 👩 | **女优浏览** | 网格视图带头像，按名字或代号搜索 |
| 🖼️ | **图片预览** | 通过 Kitty 图形协议在终端内显示真实图片 |
| 🧲 | **智能磁链** | 自动排序：字幕 > 高清 > 文件大小，一键获取 |
| ✅ | **多选** | 跨搜索选择，批量导出磁力链接 |
| 📋 | **历史 & 收藏** | SQLite 持久化，封面图片缓存 |
| ☁️ | **115 网盘** | 扫码登录，推送磁力到 115 离线下载 |
| ⚙️ | **可配置** | 图片开关、排序偏好，全部存 SQLite |
| 🎨 | **Catppuccin Frappé** | 精美护眼配色主题 |

## 🚀 安装

**macOS / Linux：**

```bash
curl -fsSL https://raw.githubusercontent.com/hyperq/jav/master/install.sh | sh
```

**Windows (PowerShell)：**

```powershell
irm https://raw.githubusercontent.com/hyperq/jav/master/install.ps1 | iex
```

**从源码编译：**

```bash
git clone https://github.com/hyperq/jav.git
cd jav
cargo build --release
cp target/release/jav ~/.local/bin/
```

## 📖 使用

```bash
# 直接运行
jav

# 使用代理
jav --proxy socks5://127.0.0.1:1080

# 自定义站点
jav --base https://www.javbus.com
```

## ⌨️ 快捷键

### 🔍 搜索

| 按键 | 功能 |
|------|------|
| `f` | 搜索影片 |
| `F` | 搜索女优 |
| `S` | 女优代号直达（如 `okq`） |
| `N` | 跳转到指定页 |

### 🧭 导航

| 按键 | 功能 |
|------|------|
| `j` / `k` | 上 / 下 |
| `h` / `l` | 左 / 右（网格列） |
| `Enter` | 打开详情 |
| `Tab` | 切换面板 |
| `n` | 加载更多 |
| `滚轮` | 鼠标滚动 |
| `点击` | 点击选中 |

### 📦 操作

| 按键 | 功能 |
|------|------|
| `空格` | 切换选中 |
| `a` | 全选/取消全选 |
| `e` | 导出磁力链接到文件 |
| `s` | 收藏/取消收藏 |
| `g` | 获取最佳磁力链接 |

### ☁️ 115 网盘

| 按键 | 功能 |
|------|------|
| `d` | 推送当前到 115 离线下载 |
| `D` | 批量推送选中到 115 |
| `L` | 扫码登录 |

### 🛠️ 其他

| 按键 | 功能 |
|------|------|
| `c` | 设置 |
| `~` | 显示/隐藏日志 |
| `q` | 退出 |

## 📁 数据

所有数据存储在 `~/.jav/`：

```
~/.jav/
├── data.db          # SQLite：历史、收藏、配置、磁链缓存
├── cache/           # 图片缓存（缩略图、封面、头像）
└── 115_cookie.json  # 115 网盘登录会话
```

## 🖥️ 终端支持

| 终端 | 图片 | 状态 |
|------|------|------|
| Ghostty | ✅ Kitty 协议 | 推荐使用 |
| Kitty | ✅ 原生支持 | 完全支持 |
| WezTerm | ✅ Kitty 协议 | 完全支持 |
| iTerm2 | ❌ | 纯文本模式 |
| Alacritty | ❌ | 纯文本模式 |
| Terminal.app | ❌ | 纯文本模式 |

> 不支持图片的终端会自动切换到纯文本模式。

## 📄 协议

MIT © [hyperq](https://github.com/hyperq)
