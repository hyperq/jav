use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui_image::{Resize, StatefulImage};

use super::app::{App, Panel, Tab, SearchMode};
use super::theme::Theme;

const CARD_HEIGHT_IMG: u16 = 10;
const CARD_HEIGHT_TEXT: u16 = 5;
const THUMB_WIDTH: u16 = 18;

fn card_height(show_img: bool) -> u16 {
    if show_img { CARD_HEIGHT_IMG } else { CARD_HEIGHT_TEXT }
}

fn clear_popup(f: &mut Frame, popup: Rect, _area: Rect) {
    f.render_widget(Clear, popup);
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let t = &app.theme.clone();
    let area = f.area();

    let mut v_constraints = vec![Constraint::Min(10), Constraint::Length(1)];
    if app.show_logs {
        v_constraints.insert(1, Constraint::Length(7));
    }
    let main = Layout::vertical(v_constraints).split(area);

    let panels = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Percentage(55),
    ]).split(main[0]);

    draw_left(f, app, panels[0]);
    draw_right(f, app, panels[1]);

    if app.show_logs {
        draw_logs(f, app, main[1]);
        draw_status(f, app, main[2]);
    } else {
        draw_status(f, app, main[1]);
    }

    if app.show_search {
        draw_search_popup(f, app, area);
    }
    if app.show_config {
        draw_config_popup(f, app, area);
    }
    if let Some((done, total)) = app.export_progress {
        draw_progress_popup(f, t, area, done, total, "📦 导出磁力链接");
    }
    if let Some((done, total)) = app.cloud_dl_progress {
        draw_progress_popup(f, t, area, done, total, "☁️ 115 云下载");
    }
    if app.show_qr_login {
        draw_qr_login_popup(f, app, area);
    }
    if app.show_download_input {
        draw_download_popup(f, app, area);
    }
    if app.show_page_jump {
        draw_page_jump_popup(f, app, area);
    }
    if app.show_player_picker {
        draw_player_picker(f, app, area);
    }
    // toast notification (auto-dismiss after 3s)
    // always clear the toast row to prevent ghost remnants from previous toasts
    let toast_row = Rect::new(0, area.height / 2 - 1, area.width, 3);
    if let Some((ref msg, instant)) = app.toast {
        if instant.elapsed() < std::time::Duration::from_secs(3) {
            let text_w: u16 = msg.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum();
            let w = (text_w + 4).max(16).min(area.width.saturating_sub(4));
            let popup = Rect::new((area.width.saturating_sub(w)) / 2, toast_row.y, w, 3);
            f.render_widget(Clear, toast_row);
            let block = Block::bordered()
                .border_style(Style::default().fg(t.peach))
                .style(Style::default().bg(t.base));
            let inner = block.inner(popup);
            f.render_widget(block, popup);
            f.render_widget(Paragraph::new(msg.as_str()).fg(t.text).alignment(Alignment::Center), inner);
        }
    }
}

fn draw_page_jump_popup(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme.clone();
    let w = 35u16.min(area.width.saturating_sub(4));
    let h = 5u16;
    let popup = Rect::new((area.width - w) / 2, (area.height - h) / 2, w, h);
    clear_popup(f, popup, area);
    let block = Block::bordered()
        .title(" 跳转页码 (Enter确认) ")
        .border_style(Style::default().fg(t.sapphire))
        .style(Style::default().bg(t.base));
    let inner = block.inner(popup);
    f.render_widget(block, popup);
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);
    f.render_widget(Paragraph::new(format!(" 当前第{}页，输入页码：", app.page)).fg(t.overlay0), rows[0]);
    f.render_widget(Paragraph::new(format!(" {}▌", app.page_jump_input)).fg(t.text).bold(), rows[1]);
}

fn draw_progress_popup(f: &mut Frame, t: &Theme, area: Rect, done: usize, total: usize, title: &str) {
    let w = 40u16.min(area.width.saturating_sub(4));
    let h = 5u16;
    let popup = Rect::new((area.width - w) / 2, (area.height - h) / 2, w, h);

    let block = Block::bordered()
        .title(format!(" {title} "))
        .border_style(Style::default().fg(t.peach))
        .style(Style::default().bg(t.base));
    let inner = block.inner(popup);
    clear_popup(f, popup, area);
    f.render_widget(block, popup);

    let pct = if total > 0 { done * 100 / total } else { 0 };
    let bar_width = (inner.width as usize).saturating_sub(2);
    let filled = bar_width * pct / 100;
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

    let lines = vec![
        Line::styled(format!(" {done}/{total} ({pct}%)"), Style::default().fg(t.text)),
        Line::styled(format!(" {bar}"), Style::default().fg(t.peach)),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

// --- Left Panel ---

fn draw_left(f: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme.clone();
    let border_style = if app.panel == Panel::Left {
        Style::default().fg(t.mauve)
    } else {
        Style::default().fg(t.overlay0)
    };

    // tabs on the border (like right panel title)
    let tab_labels = vec!["🔍影片", "👩女优", "📋历史", "⭐收藏"];
    let active = match app.tab { Tab::Movies => 0, Tab::Actresses => 1, Tab::History => 2, Tab::Favorites => 3 };
    let tab_title: String = tab_labels.iter().enumerate().map(|(i, &t)| {
        if i == active { format!(" [{t}] ") } else { format!(" {t} ") }
    }).collect::<Vec<_>>().join("│");

    let block = Block::bordered().title(tab_title).border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    match app.tab {
        Tab::Actresses => draw_actress_list(f, app, inner),
        _ => draw_movie_list(f, app, inner),
    }
}

fn draw_movie_list(f: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme.clone();
    let movies = app.current_movies().to_vec();
    let cur = app.cursor();

    if app.loading && movies.is_empty() {
        f.render_widget(
            Paragraph::new("⏳ 加载中...").fg(t.yellow).alignment(Alignment::Center),
            Rect::new(area.x, area.y + area.height / 2, area.width, 1),
        );
        return;
    }
    if movies.is_empty() {
        let hint = match app.tab {
            Tab::History => "📭 暂无历史 (f:搜索影片 F:搜女优 S:女优代号)",
            Tab::Favorites => "💫 暂无收藏 (列表页按 s 收藏)",
            _ => "按 f 搜索影片 | F 搜索女优 | S 女优代号直达",
        };
        f.render_widget(Paragraph::new(hint).fg(t.overlay0).alignment(Alignment::Center),
            Rect::new(area.x, area.y + area.height / 2, area.width, 1));
        return;
    }

    // info bar at top
    let split = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);
    {
        let mut spans: Vec<Span> = Vec::new();
        let info = match app.tab {
            Tab::Movies => &app.search_info,
            Tab::History => &format!("历史"),
            Tab::Favorites => &format!("收藏"),
            _ => &String::new(),
        };
        if !info.is_empty() {
            spans.push(Span::styled(format!(" {info} "), Style::default().fg(t.sapphire)));
            spans.push(Span::styled("│", Style::default().fg(t.surface2)));
        }
        spans.push(Span::styled(format!(" {}条", movies.len()), Style::default().fg(t.text)));
        spans.push(Span::styled(format!(" #{}", cur + 1), Style::default().fg(t.peach)));
        if app.tab == Tab::Movies {
            spans.push(Span::styled(format!(" P{}", app.page), Style::default().fg(t.overlay0)));
            if app.has_next {
                spans.push(Span::styled(" │ n:更多 ▼", Style::default().fg(t.overlay0)));
            }
        }
        if app.loading { spans.push(Span::styled(" ⏳", Style::default().fg(t.yellow))); }
        if let Some((done, total)) = app.export_progress {
            let pct = if total > 0 { done * 100 / total } else { 0 };
            spans.push(Span::styled(format!(" │ 📦{done}/{total}({pct}%)"), Style::default().fg(t.peach)));
        }
        let sel = app.selected_numbers.len();
        if sel > 0 {
            spans.push(Span::styled(format!(" │ ✅{sel}"), Style::default().fg(t.green)));
        }
        f.render_widget(Paragraph::new(Line::from(spans)).bg(t.mantle), split[0]);
    }
    let area = split[1];

    // 2-column grid layout
    let ch = card_height(app.config.show_thumbnails);
    let col_width = area.width / 2;
    let rows_per_page = (area.height / ch) as usize;
    let items_per_page = rows_per_page * 2;

    let cursor_row = cur / 2;
    let scroll_row = if cursor_row >= rows_per_page { cursor_row - rows_per_page + 1 } else { 0 };
    let offset = scroll_row * 2;

    // trigger image loading
    if app.config.show_thumbnails {
        let visible_items: Vec<(String, String)> = movies[offset..]
            .iter().take(items_per_page + 2)
            .map(|m| (m.number.clone(), m.cover.clone()))
            .collect();
        for (number, cover) in &visible_items {
            if !cover.is_empty() {
                app.do_fetch_image(cover, number);
            }
        }
    }

    // render in 2 columns
    let items_to_render: Vec<(usize, Movie, bool)> = movies[offset..]
        .iter().take(items_per_page + 2).enumerate()
        .map(|(i, m)| (offset + i, m.clone(), app.selected_numbers.contains(&m.number)))
        .collect();

    for (i, (idx, movie, multi_selected)) in items_to_render.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let y = area.y + (row as u16) * ch;
        let x = area.x + (col as u16) * col_width;

        if y + ch > area.y + area.height { break; }

        let card_area = Rect::new(x, y, col_width, ch);
        let cursor_on = *idx == app.cursor() && app.panel == Panel::Left;
        draw_movie_card(f, app, movie, card_area, cursor_on, *multi_selected);
    }

}

fn draw_movie_card(f: &mut Frame, app: &mut App, movie: &crate::scraper::Movie, area: Rect, cursor_on: bool, multi_selected: bool) {
    let t = &app.theme.clone();
    if cursor_on {
        f.render_widget(Block::default().style(Style::default().bg(t.surface0)), area);
    }

    let thumb_w = if app.config.show_thumbnails { THUMB_WIDTH } else { 0 };
    let chunks = Layout::horizontal([
        Constraint::Length(thumb_w),
        Constraint::Min(1),
    ]).split(area);

    // thumbnail
    if app.config.show_thumbnails {
        let number = movie.number.clone();
        let thumb_rect = chunks[0];
        if let Some(proto) = app.image_cache.get_mut(&number) {
            let img = StatefulImage::default().resize(Resize::Crop(None));
            f.render_stateful_widget(img, thumb_rect, proto);
        } else {
            f.render_widget(
                Paragraph::new("📷").fg(t.overlay0).alignment(Alignment::Center),
                Rect::new(thumb_rect.x, thumb_rect.y + thumb_rect.height / 2, thumb_rect.width, 1),
            );
        }
    }

    // text info
    let info_area = if app.config.show_thumbnails {
        Rect::new(chunks[1].x + 1, chunks[1].y, chunks[1].width.saturating_sub(2), chunks[1].height)
    } else {
        Rect::new(chunks[1].x, chunks[1].y, chunks[1].width, chunks[1].height)
    };
    let info_chunks = if app.config.show_thumbnails {
        Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ]).split(info_area)
    } else {
        Layout::vertical([
            Constraint::Length(2), // title (2 lines)
            Constraint::Length(1), // tags
            Constraint::Length(1), // number / date
        ]).split(info_area)
    };

    // title
    let cursor_mark = " ";
    let title_style = if cursor_on { Style::default().fg(t.mauve).bold() } else { Style::default().fg(t.text) };
    let title_lines = info_chunks[0].height.max(1) as usize;
    let avail_w = info_area.width as usize;
    let max_title_chars = avail_w.saturating_sub(2) * title_lines;
    let title_text = format!("{cursor_mark}{}", truncate_str(&movie.title, max_title_chars));
    f.render_widget(Paragraph::new(title_text).style(title_style).wrap(Wrap { trim: true }), info_chunks[0]);

    // tags (filter out ⭐)
    let real_tags: Vec<&String> = movie.tags.iter().filter(|tag| !tag.contains("⭐")).collect();

    // tags on separate row (both modes)
    let tag_spans: Vec<Span> = real_tags.iter().map(|tag| {
        let style = if tag.contains("高清") { Style::default().fg(t.green).bold() }
                    else { Style::default().fg(t.red).bold() };
        Span::styled(format!("[{tag}] "), style)
    }).collect();
    if !tag_spans.is_empty() {
        f.render_widget(Paragraph::new(Line::from(tag_spans)), info_chunks[1]);
    }

    // number / date + select & favorite marks + tags (inline when no thumbnails)
    let select_icon = if multi_selected { "✅" } else { "" };
    let fav_icon = if movie.tags.iter().any(|t| t.contains("⭐")) { "⭐" } else { "" };
    let date_str = if movie.date.is_empty() { "" } else { &movie.date };

    let mut num_spans: Vec<Span> = Vec::new();
    if !select_icon.is_empty() { num_spans.push(Span::styled(select_icon, Style::default().fg(t.green))); }
    if !fav_icon.is_empty() { num_spans.push(Span::styled(fav_icon, Style::default().fg(t.yellow))); }
    num_spans.push(Span::styled(format!(" {}", movie.number), Style::default().fg(t.overlay0)));
    if !date_str.is_empty() {
        num_spans.push(Span::styled(format!(" / {date_str}"), Style::default().fg(t.overlay0)));
    }
    f.render_widget(Paragraph::new(Line::from(num_spans)), info_chunks[2]);
}

fn draw_actress_list(f: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme.clone();
    if app.loading && app.actresses.is_empty() {
        f.render_widget(
            Paragraph::new("⏳ 加载中...").fg(t.yellow).alignment(Alignment::Center),
            Rect::new(area.x, area.y + area.height / 2, area.width, 1),
        );
        return;
    }
    if app.actresses.is_empty() {
        f.render_widget(
            Paragraph::new("按 F 搜索女优 | S 输入女优代号直达").fg(t.overlay0).alignment(Alignment::Center),
            Rect::new(area.x, area.y + area.height / 2, area.width, 1),
        );
        return;
    }

    // flex grid: 3 columns, each cell = avatar(top) + name(bottom)
    let cols_per_row: usize = 3;
    let cell_w: u16 = area.width / cols_per_row as u16;
    let cell_h: u16 = 12; // avatar ~9 rows + name 2 rows + spacing
    let rows_per_page = (area.height / cell_h).max(1) as usize;
    let items_per_page = cols_per_row * rows_per_page;

    let cursor_row = app.cursor() / cols_per_row;
    let scroll_row = if cursor_row >= rows_per_page { cursor_row - rows_per_page + 1 } else { 0 };
    let offset = scroll_row * cols_per_row;

    // load avatars
    let visible_items: Vec<(String, String)> = app.actresses[offset..]
        .iter().take(items_per_page + cols_per_row)
        .map(|a| (format!("actress_{}", a.code), a.avatar.clone()))
        .collect();
    for (key, url) in &visible_items {
        app.do_fetch_image(url, key);
    }

    // render grid
    let items: Vec<(usize, crate::scraper::Actress)> = app.actresses[offset..]
        .iter().take(items_per_page + cols_per_row).enumerate()
        .map(|(i, a)| (offset + i, a.clone()))
        .collect();

    for (i, (idx, actress)) in items.iter().enumerate() {
        let col = i % cols_per_row;
        let row = i / cols_per_row;
        let x = area.x + (col as u16) * cell_w;
        let y = area.y + (row as u16) * cell_h;

        if y + cell_h > area.y + area.height { break; }

        let cell = Rect::new(x, y, cell_w, cell_h);
        let selected = *idx == app.cursor() && app.panel == Panel::Left;

        if selected {
            f.render_widget(Block::default().style(Style::default().bg(t.surface0)), cell);
        }

        // avatar area (centered square)
        let avatar_h = cell_h - 2;
        // make avatar width proportional to height for square-ish display
        // font ratio: each cell is ~17w x 44h, so to get square: w_cols = h_rows * 44/17 ≈ h*2.5
        let avatar_w = (avatar_h as u32 * 44 / 17).min(cell_w.saturating_sub(2) as u32) as u16;
        let avatar_x = x + (cell_w - avatar_w) / 2; // center horizontally
        let avatar_rect = Rect::new(avatar_x, y, avatar_w, avatar_h);
        let avatar_key = format!("actress_{}", actress.code);
        if let Some(proto) = app.image_cache.get_mut(&avatar_key) {
            let img = StatefulImage::default().resize(Resize::Fit(None));
            f.render_stateful_widget(img, avatar_rect, proto);
        } else {
            f.render_widget(
                Paragraph::new("👩").fg(t.overlay0).alignment(Alignment::Center),
                Rect::new(x, y + avatar_h / 2, cell_w, 1),
            );
        }

        // name + code (bottom, centered)
        let name_style = if selected { Style::default().fg(t.mauve).bold() } else { Style::default().fg(t.text) };
        let name_area = Rect::new(x, y + avatar_h, cell_w, 2);
        let name = truncate_str(&actress.name, cell_w as usize - 2);
        let lines = vec![
            Line::styled(name, name_style),
            Line::styled(actress.code.clone(), Style::default().fg(t.overlay0)),
        ];
        f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), name_area);
    }
}

// --- Right Panel ---

fn draw_right(f: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme.clone();
    let border_style = if app.panel == Panel::Right {
        Style::default().fg(t.mauve)
    } else {
        Style::default().fg(t.overlay0)
    };

    let title = app.selected_movie.as_ref()
        .map(|m| format!("🎬 {} - {}", m.number, truncate_str(&m.title, 40)))
        .unwrap_or_else(|| "🎬 影片详情".into());

    let block = Block::bordered().title(title).border_style(border_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.selected_movie.is_none() {
        f.render_widget(
            Paragraph::new("👈 选择影片查看详情").fg(t.overlay0).alignment(Alignment::Center),
            Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1),
        );
        return;
    }

    // build all content lines, then apply scroll
    let mut lines: Vec<Line> = Vec::new();

    // cover image (only if enabled and kitty supported)
    let show_cover = app.config.show_cover && app.config.kitty_supported;
    let cover_h = if show_cover {
        ((inner.width as u32) * 26 / 100).max(4).min(inner.height.saturating_sub(8) as u32) as u16
    } else {
        0
    };
    let content_area = Layout::vertical([
        Constraint::Length(cover_h),
        Constraint::Min(1),
    ]).split(inner);

    if show_cover {
        let cover_key = app.selected_movie.as_ref().map(|m| format!("cover_{}", m.number)).unwrap_or_default();
        if let Some(proto) = app.image_cache.get_mut(&cover_key) {
            let img = StatefulImage::default().resize(Resize::Crop(None));
            f.render_stateful_widget(img, content_area[0], proto);
        } else {
            f.render_widget(
                Paragraph::new("⏳ 加载封面...").fg(t.overlay0).alignment(Alignment::Center),
                Rect::new(content_area[0].x, content_area[0].y + content_area[0].height / 2, content_area[0].width, 1),
            );
        }
    }

    // scrollable content below cover
    let w = content_area[1].width as usize;

    // metadata
    if let Some(detail) = &app.movie_detail {
        let genres_str = detail.genres.join(", ");
        let actresses_str = detail.actresses.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ");
        lines.push(Line::styled(format!(" 📅 {} | ⏱ {} | 🏭 {}", detail.date, detail.duration, detail.maker), Style::default().fg(t.text)));
        lines.push(Line::styled(format!(" 📦 {}", detail.publisher), Style::default().fg(t.overlay0)));
        // genres & actresses: wrap to multiple lines
        let line_w = w.saturating_sub(5);
        for (prefix, text, color) in [
            (" 🏷  ", &genres_str, t.green),
            (" 👩 ", &actresses_str, t.sapphire),
        ] {
            let wrapped = wrap_text(text, line_w);
            for (i, chunk) in wrapped.iter().enumerate() {
                let p = if i == 0 { prefix } else { "     " };
                lines.push(Line::styled(format!("{p}{chunk}"), Style::default().fg(color)));
            }
        }
    } else {
        lines.push(Line::styled(" ⏳ 加载详情...", Style::default().fg(t.overlay0)));
    }

    lines.push(Line::raw(""));

    // magnets - sorted by caption > HD > size
    let mut sorted_mags: Vec<&crate::scraper::Magnet> = app.magnets.iter().collect();
    sorted_mags.sort_by(|a, b| {
        b.caption.cmp(&a.caption)
            .then_with(|| {
                let ha = a.size.contains("HD");
                let hb = b.size.contains("HD");
                hb.cmp(&ha)
            })
            .then_with(|| {
                let sa = parse_size_mb(&a.size);
                let sb = parse_size_mb(&b.size);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    lines.push(Line::styled(format!(" 🧲 磁力链接 ({}) | g:获取最佳", app.magnets.len()), Style::default().fg(t.yellow)));
    if sorted_mags.is_empty() {
        lines.push(Line::styled(" ⏳ 加载磁链中...", Style::default().fg(t.overlay0)));
    } else {
        for m in &sorted_mags {
            let badge = if m.caption { "✅字幕 " } else { "" };
            let hd = if m.size.contains("HD") { "🎬HD " } else { "" };
            lines.push(Line::styled(
                format!("  {badge}{hd}[{}]", m.size),
                Style::default().fg(t.sapphire),
            ));
            lines.push(Line::styled(
                format!("   {}", truncate_str(&m.link, w.saturating_sub(4))),
                Style::default().fg(t.blue),
            ));
        }
    }

    // render scrollable content
    let scroll = app.right_scroll;
    let paragraph = Paragraph::new(lines).scroll((scroll, 0));
    f.render_widget(paragraph, content_area[1]);
}

// --- Bottom panels ---

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme.clone();
    let block = Block::bordered().title("📝 日志 (~隐藏)").border_style(Style::default().fg(t.yellow));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible = inner.height as usize;
    let start = app.logs.len().saturating_sub(visible);
    let lines: Vec<Line> = app.logs[start..].iter().map(|l| Line::raw(l.as_str())).collect();
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme.clone();
    let sel_count = app.selected_numbers.len();
    let sel_info = if sel_count > 0 { format!(" | 已选{sel_count}条") } else { String::new() };

    let keys: Vec<Span> = vec![
        Span::styled(" f", Style::default().fg(t.sapphire).bold()),
        Span::raw(":搜索 "),
        Span::styled("F", Style::default().fg(t.sapphire).bold()),
        Span::raw(":女优 "),
        Span::styled("S", Style::default().fg(t.sapphire).bold()),
        Span::raw(":代号 "),
        Span::styled("空格", Style::default().fg(t.sapphire).bold()),
        Span::raw(":选择 "),
        Span::styled("a", Style::default().fg(t.sapphire).bold()),
        Span::raw(":全选 "),
        Span::styled("e", Style::default().fg(t.sapphire).bold()),
        Span::raw(":导出 "),
        Span::styled("s", Style::default().fg(t.sapphire).bold()),
        Span::raw(":收藏 "),
        Span::styled("g", Style::default().fg(t.sapphire).bold()),
        Span::raw(":磁链 "),
        Span::styled("n", Style::default().fg(t.sapphire).bold()),
        Span::raw(":更多 "),
        Span::styled("d", Style::default().fg(t.teal).bold()),
        Span::raw(":115↓ "),
        Span::styled("D", Style::default().fg(t.teal).bold()),
        Span::raw(":115批量↓ "),
        Span::styled("L", Style::default().fg(t.teal).bold()),
        Span::raw(":115登录 "),
        Span::styled("c", Style::default().fg(t.sapphire).bold()),
        Span::raw(":设置 "),
        Span::styled("Tab", Style::default().fg(t.sapphire).bold()),
        Span::raw(":面板 "),
        Span::styled("q", Style::default().fg(t.sapphire).bold()),
        Span::raw(":退出"),
        Span::styled(&sel_info, Style::default().fg(t.yellow)),
        if let Some(ref q) = app.cloud115_quota {
            Span::styled(format!(" │ ☁️{}/{}", q.quota, q.total), Style::default().fg(t.green))
        } else if app.cloud115.is_some() {
            Span::styled(" │ ☁️115", Style::default().fg(t.green))
        } else {
            Span::styled(" │ 115未登录", Style::default().fg(t.overlay0))
        },
    ];

    f.render_widget(
        Paragraph::new(Line::from(keys)).style(Style::default().bg(t.crust)),
        area,
    );
}

// --- Search popup ---

fn draw_search_popup(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme.clone();
    let mode_str = match app.search_mode {
        SearchMode::Movie => "🔍 搜索影片",
        SearchMode::Actress => "👩 搜索女优",
        SearchMode::StarCode => "📌 女优代号直达",
    };
    let title = format!(" {mode_str} (Enter / Esc) ");

    // width: at least title width + border, capped to screen
    let w = 50u16.min(area.width.saturating_sub(4));
    let h = 7u16;
    let popup = Rect::new((area.width.saturating_sub(w)) / 2, (area.height.saturating_sub(h)) / 2, w, h);

    clear_popup(f, popup, area);

    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(t.mauve))
        .style(Style::default().bg(t.mantle));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(inner);

    let hint = match app.search_mode {
        SearchMode::Movie => "输入番号或关键词：",
        SearchMode::Actress => "输入女优名：",
        SearchMode::StarCode => "输入女优代号 (如 okq)：",
    };
    f.render_widget(Paragraph::new(hint).fg(t.overlay0), rows[0]);
    f.render_widget(Paragraph::new(format!(" {}▌", app.search_input)).fg(t.text).bold(), rows[1]);

    if matches!(app.search_mode, SearchMode::Movie | SearchMode::Actress) {
        let (cs, us) = if app.uncensored {
            (Style::default().fg(t.overlay0), Style::default().fg(t.mauve).bold())
        } else {
            (Style::default().fg(t.mauve).bold(), Style::default().fg(t.overlay0))
        };
        let line = Line::from(vec![
            Span::raw(" [Tab] "), Span::styled("有码", cs), Span::raw(" / "), Span::styled("无码", us),
        ]);
        f.render_widget(Paragraph::new(line), rows[2]);
    }
}

fn draw_config_popup(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme.clone();
    f.render_widget(Clear, area);
    f.render_widget(Block::default().style(Style::default().bg(t.crust)), area);

    let w = 48u16.min(area.width.saturating_sub(4));
    let h = 17u16;
    let popup = Rect::new((area.width - w) / 2, (area.height - h) / 2, w, h);

    let block = Block::bordered()
        .title(" ⚙️ 设置 (数字键切换 / Esc关闭) ")
        .border_style(Style::default().fg(t.sapphire))
        .style(Style::default().bg(t.base));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let on = Style::default().fg(t.green).bold();
    let off = Style::default().fg(t.red);
    let cfg = &app.config;

    let cache_dir = format!("{}/.jav/cache", std::env::var("HOME").unwrap_or_default());
    let cache_size = std::fs::read_dir(&cache_dir).ok()
        .map(|entries| entries.filter_map(|e| e.ok()).filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum::<u64>())
        .unwrap_or(0);
    let cache_mb = cache_size as f64 / 1024.0 / 1024.0;

    let kitty_label = if cfg.kitty_supported { "✅" } else { "❌不支持" };

    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(" 1", Style::default().fg(t.sapphire).bold()),
            Span::raw(". 列表显示图片  "),
            Span::styled(if cfg.show_thumbnails { "[开]" } else { "[关]" }, if cfg.show_thumbnails { on } else { off }),
        ]),
        Line::from(vec![
            Span::styled(" 2", Style::default().fg(t.sapphire).bold()),
            Span::raw(". 详情显示封面  "),
            Span::styled(if cfg.show_cover { "[开]" } else { "[关]" }, if cfg.show_cover { on } else { off }),
        ]),
        Line::from(vec![
            Span::styled(" 3", Style::default().fg(t.sapphire).bold()),
            Span::raw(". 优先字幕      "),
            Span::styled(if cfg.prefer_caption { "[开]" } else { "[关]" }, if cfg.prefer_caption { on } else { off }),
        ]),
        Line::from(vec![
            Span::styled(" 4", Style::default().fg(t.sapphire).bold()),
            Span::raw(". 优先高清      "),
            Span::styled(if cfg.prefer_hd { "[开]" } else { "[关]" }, if cfg.prefer_hd { on } else { off }),
        ]),
        Line::from(vec![
            Span::styled(" 5", Style::default().fg(t.sapphire).bold()),
            Span::raw(". 按大小排序    "),
            Span::styled(if cfg.sort_by_size { "[开]" } else { "[关]" }, if cfg.sort_by_size { on } else { off }),
        ]),
        Line::from(vec![
            Span::styled(" 6", Style::default().fg(t.red).bold()),
            Span::raw(". 清理图片缓存  "),
            Span::styled(format!("[{:.1}MB]", cache_mb), Style::default().fg(t.peach)),
        ]),
        Line::from(vec![
            Span::styled(" 7", Style::default().fg(t.sapphire).bold()),
            Span::raw(". 播放器        "),
            Span::styled(
                format!("[{}]", if cfg.player.is_empty() { "未设置" } else { &cfg.player }),
                Style::default().fg(t.peach),
            ),
        ]),
        Line::from(vec![
            Span::styled(" 8", Style::default().fg(t.sapphire).bold()),
            Span::raw(". 主题          "),
            Span::styled(format!("[{}]", cfg.theme_mode.label()), Style::default().fg(t.peach)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!(" Kitty协议: {kitty_label}"), Style::default().fg(t.overlay0)),
            Span::raw(" | "),
            Span::styled("~/.jav/", Style::default().fg(t.overlay0)),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_qr_login_popup(f: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme.clone();
    let font = app.picker.font_size();

    // calculate QR image size in terminal cells
    // QR was resized to 24*font_h pixels square in App, figure out how many rows/cols that is
    let qr_px = 24u16 * font.1;
    let img_rows = qr_px / font.1;                  // ≈24
    let img_cols = qr_px / font.0;                   // wider because font_w < font_h

    // popup = border(2) + status(1) + image + hint(1)
    let popup_h = (img_rows + 4).min(area.height.saturating_sub(2));
    let popup_w = (img_cols + 4).min(area.width.saturating_sub(2));
    let popup = Rect::new(
        (area.width.saturating_sub(popup_w)) / 2,
        (area.height.saturating_sub(popup_h)) / 2,
        popup_w, popup_h,
    );

    clear_popup(f, popup, area);

    let cloud_status = if app.cloud115.is_some() { " [已登录]" } else { "" };
    let block = Block::bordered()
        .title(format!(" ☁️ 115{cloud_status} (Esc/r) "))
        .border_style(Style::default().fg(t.teal))
        .style(Style::default().bg(t.base));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ]).split(inner);

    let status_style = if app.qr_status_text.contains("成功") {
        Style::default().fg(t.green).bold()
    } else if app.qr_status_text.contains("失败") || app.qr_status_text.contains("过期") {
        Style::default().fg(t.red)
    } else {
        Style::default().fg(t.yellow)
    };
    f.render_widget(
        Paragraph::new(format!(" {}", app.qr_status_text)).style(status_style),
        rows[0],
    );

    // QR image: pad 1 col each side so image doesn't touch border
    let qr_area = Rect::new(
        rows[1].x + 1,
        rows[1].y,
        rows[1].width.saturating_sub(2),
        rows[1].height,
    );
    if let Some(proto) = app.image_cache.get_mut("qr_115") {
        let img = StatefulImage::default().resize(Resize::Fit(None));
        f.render_stateful_widget(img, qr_area, proto);
    } else {
        f.render_widget(
            Paragraph::new("⏳ 加载二维码...").fg(t.overlay0).alignment(Alignment::Center),
            Rect::new(qr_area.x, qr_area.y + qr_area.height / 2, qr_area.width, 1),
        );
    }

    f.render_widget(
        Paragraph::new(" 用 115 APP 扫码").fg(t.overlay0).alignment(Alignment::Center),
        rows[2],
    );
}

fn draw_player_picker(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme.clone();
    let item_count = app.player_options.len() as u16;
    let w = 40u16.min(area.width.saturating_sub(4));
    let h = (item_count + 4).min(area.height.saturating_sub(4));
    let popup = Rect::new((area.width - w) / 2, (area.height - h) / 2, w, h);

    clear_popup(f, popup, area);
    let block = Block::bordered()
        .title(" ▶️ 选择播放器 (Enter/Esc) ")
        .border_style(Style::default().fg(t.teal))
        .style(Style::default().bg(t.base));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let mut lines = vec![Line::raw("")];
    for (i, &(_value, name)) in app.player_options.iter().enumerate() {
        let selected = i == app.player_cursor;
        let prefix = if selected { " ▸ " } else { "   " };
        let style = if selected {
            Style::default().fg(t.mauve).bold()
        } else {
            Style::default().fg(t.text)
        };
        lines.push(Line::styled(format!("{prefix}{name}"), style));
    }
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_download_popup(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme.clone();
    let w = 55u16.min(area.width.saturating_sub(4));
    let h = 7u16;
    let popup = Rect::new((area.width - w) / 2, (area.height - h) / 2, w, h);

    let block = Block::bordered()
        .title(" ☁️ 115 云下载 (Enter确认 / Esc取消) ")
        .border_style(Style::default().fg(t.teal))
        .style(Style::default().bg(t.base));
    let inner = block.inner(popup);
    clear_popup(f, popup, area);
    f.render_widget(block, popup);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(inner);

    let info = if !app.pending_batch_movies.is_empty() {
        format!("🎬 批量 {} 部影片", app.pending_batch_movies.len())
    } else {
        app.selected_movie.as_ref()
            .map(|m| format!("🎬 {}", m.number))
            .unwrap_or_default()
    };
    f.render_widget(Paragraph::new(format!(" {info}")).fg(t.text), rows[0]);

    f.render_widget(
        Paragraph::new(" 下载目录ID (留空=默认目录):").fg(t.overlay0),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(format!(" {}▌", app.download_dir)).fg(t.text).bold(),
        rows[2],
    );
}

use crate::scraper::Movie;

fn wrap_text(s: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 { return vec![s.to_string()]; }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars { return vec![s.to_string()]; }
    chars.chunks(max_chars).map(|c| c.iter().collect()).collect()
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars { return s.to_string(); }
    let truncated: String = chars[..max_chars].iter().collect();
    format!("{truncated}...")
}

fn parse_size_mb(s: &str) -> f64 {
    let s = s.trim();
    let num: f64 = s.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect::<String>().parse().unwrap_or(0.0);
    if s.contains('G') { num * 1024.0 } else { num }
}
