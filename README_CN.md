# JAV TUI

JavBus 终端客户端 — 搜索影片、浏览女优、管理磁力链接，支持图片预览。

![Rust](https://img.shields.io/badge/rust-2024-orange) ![License](https://img.shields.io/badge/license-MIT-blue)

[English](README.md)

## 功能

- **影片搜索** — 按关键词、番号、女优代号搜索
- **女优浏览** — 网格视图带头像，按名字搜索
- **图片预览** — Kitty 图形协议（Ghostty/Kitty/WezTerm）
- **磁力链接** — 智能排序（字幕 > 高清 > 文件大小），批量导出
- **多选** — 空格选择，批量导出磁力到文件
- **历史 & 收藏** — SQLite 持久化，封面缓存
- **115 网盘** — 扫码登录，直接推送磁力到 115 离线下载
- **可配置** — 缩略图、封面、排序偏好存储在 SQLite
- **Catppuccin Frappé** — 精美配色主题

## 安装

```bash
cargo build --release
cp target/release/jav ~/.local/bin/
```

## 使用

```bash
# 直接运行
jav

# 使用代理
jav --proxy socks5://127.0.0.1:1080

# 自定义站点
jav --base https://www.javbus.com
```

## 快捷键

| 按键 | 功能 |
|------|------|
| `f` | 搜索影片 |
| `F` | 搜索女优 |
| `S` | 女优代号直达（如 `okq`） |
| `j/k` | 上下导航 |
| `h/l` | 左右导航（网格列） |
| `Enter` | 打开详情 |
| `Tab` | 切换面板（列表/详情） |
| `n` | 加载更多（下一页） |
| `N` | 跳转到指定页 |
| `空格` | 多选切换 |
| `a` | 全选/取消全选 |
| `e` | 导出选中的磁力链接 |
| `s` | 收藏/取消收藏 |
| `g` | 获取最佳磁力链接 |
| `d` | 推送到 115 离线下载 |
| `D` | 批量推送选中到 115 |
| `L` | 115 扫码登录 |
| `c` | 设置 |
| `~` | 显示/隐藏日志面板 |
| `q` | 退出 |
| 滚轮 | 鼠标滚动 |
| 点击 | 点击列表项/标签 |

## 数据

所有数据存储在 `~/.jav/`：

| 文件 | 内容 |
|------|------|
| `data.db` | SQLite：历史、收藏、配置、磁链缓存 |
| `cache/` | 图片缓存（缩略图、封面、头像） |
| `115_cookie.json` | 115 网盘登录 Cookie |

## 环境要求

- Rust 1.70+
- 支持 Kitty 图形协议的终端（Ghostty、Kitty、WezTerm）可显示图片
- 不支持的终端自动切换到纯文本模式

## 协议

MIT
