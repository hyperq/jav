use crossterm::event::KeyCode;
use super::app::{App, Panel, Tab, SearchMode};

pub fn handle_key(app: &mut App, key: KeyCode) -> bool {
    // config popup
    if app.show_config {
        match key {
            KeyCode::Char('1') => { app.config.show_thumbnails = !app.config.show_thumbnails && app.config.kitty_supported; app.config.save(&app.store); }
            KeyCode::Char('2') => { app.config.show_cover = !app.config.show_cover && app.config.kitty_supported; app.config.save(&app.store); }
            KeyCode::Char('3') => { app.config.prefer_caption = !app.config.prefer_caption; app.config.save(&app.store); }
            KeyCode::Char('4') => { app.config.prefer_hd = !app.config.prefer_hd; app.config.save(&app.store); }
            KeyCode::Char('5') => { app.config.sort_by_size = !app.config.sort_by_size; app.config.save(&app.store); }
            KeyCode::Char('6') => { app.clear_image_cache(); }
            KeyCode::Esc | KeyCode::Char('c') => { app.show_config = false; }
            _ => {}
        }
        return false;
    }

    // qr login popup
    if app.show_qr_login {
        match key {
            KeyCode::Esc => { app.show_qr_login = false; }
            KeyCode::Char('r') => { app.start_qr_login(); } // refresh QR
            _ => {}
        }
        return false;
    }

    // download dir input popup
    if app.show_download_input {
        match key {
            KeyCode::Esc => {
                app.show_download_input = false;
                app.pending_magnets.clear();
                app.pending_batch_movies.clear();
            }
            KeyCode::Enter => { app.confirm_download(); }
            KeyCode::Backspace => { app.download_dir.pop(); }
            KeyCode::Char(c) => { app.download_dir.push(c); }
            _ => {}
        }
        return false;
    }

    // page jump popup
    if app.show_page_jump {
        match key {
            KeyCode::Esc => { app.show_page_jump = false; }
            KeyCode::Enter => {
                if let Ok(p) = app.page_jump_input.parse::<usize>() {
                    if p >= 1 {
                        app.page = p;
                        app.loading = true;
                        app.do_fetch_next_page();
                    }
                }
                app.show_page_jump = false;
            }
            KeyCode::Backspace => { app.page_jump_input.pop(); }
            KeyCode::Char(c) if c.is_ascii_digit() => { app.page_jump_input.push(c); }
            _ => {}
        }
        return false;
    }

    // search popup mode
    if app.show_search {
        match key {
            KeyCode::Esc => { app.show_search = false; }
            KeyCode::Enter => { app.execute_search(); }
            KeyCode::Tab => { app.uncensored = !app.uncensored; }
            KeyCode::Backspace => { app.search_input.pop(); }
            KeyCode::Char(c) => { app.search_input.push(c); }
            _ => {}
        }
        return false;
    }

    // normal mode
    match key {
        KeyCode::Char('q') => return true,

        // search shortcuts
        KeyCode::Char('f') | KeyCode::Char('/') => app.open_search(SearchMode::Movie),
        KeyCode::Char('F') => app.open_search(SearchMode::Actress),
        KeyCode::Char('S') => app.open_search(SearchMode::StarCode),

        // panel switch
        KeyCode::Tab => {
            app.panel = if app.panel == Panel::Left { Panel::Right } else { Panel::Left };
        }
        KeyCode::Esc => {
            app.panel = Panel::Left;
            // keep cursor position when returning to list
        }

        // left/right: navigate columns in grid
        KeyCode::Left | KeyCode::Char('h') => {
            if app.panel == Panel::Left {
                let cols = app.grid_cols();
                let cur = app.cursor();
                if cur % cols > 0 {
                    *app.cursor_mut() = cur - 1;
                }
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.panel == Panel::Left {
                let cols = app.grid_cols();
                let max = match app.tab {
                    Tab::Actresses => app.actresses.len(),
                    _ => app.current_movies().len(),
                };
                let cur = app.cursor();
                if cur % cols < cols - 1 && cur + 1 < max {
                    *app.cursor_mut() = cur + 1;
                }
            }
        }

        // navigation (grid: j/k move by cols_per_row)
        KeyCode::Char('j') | KeyCode::Down => {
            if app.panel == Panel::Left {
                let max = match app.tab {
                    Tab::Actresses => app.actresses.len(),
                    _ => app.current_movies().len(),
                };
                let step = app.grid_cols();
                let cur = app.cursor();
                if cur + step < max {
                    *app.cursor_mut() = cur + step;
                } else if cur < max.saturating_sub(1) {
                    *app.cursor_mut() = max - 1;
                }
            } else {
                app.right_scroll += 2;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.panel == Panel::Left {
                let step = app.grid_cols();
                let cur = app.cursor();
                *app.cursor_mut() = cur.saturating_sub(step);
            } else {
                app.right_scroll = app.right_scroll.saturating_sub(2);
            }
        }

        // select / enter
        KeyCode::Enter => {
            if app.panel == Panel::Left {
                app.select_current();
            }
        }

        // grab magnet for current item (smart sort)
        KeyCode::Char('g') => {
            app.grab_magnet();
        }

        // multi-select (space to toggle, move to next item)
        KeyCode::Char(' ') => {
            if app.panel == Panel::Left {
                app.toggle_select();
                let max = if app.tab == Tab::Actresses { app.actresses.len() } else { app.current_movies().len() };
                let cur = app.cursor();
                if cur + 1 < max { *app.cursor_mut() = cur + 1; }
            }
        }
        // select all
        KeyCode::Char('a') => {
            if app.panel == Panel::Left { app.select_all(); }
        }

        // export selected magnets to file
        KeyCode::Char('e') => { app.export_selected_magnets(); }

        // favorite toggle
        KeyCode::Char('s') => { app.toggle_favorite(); }

        // 115 cloud
        KeyCode::Char('d') => { app.download_detail_to_115(); }
        KeyCode::Char('D') => { app.download_selected_to_115(); }
        KeyCode::Char('L') => { app.start_qr_login(); }

        // config popup
        KeyCode::Char('c') => { app.show_config = !app.show_config; }

        // pagination
        KeyCode::Char('n') => {
            if app.has_next {
                app.page += 1;
                app.loading = true;
                if app.tab == Tab::Actresses {
                    app.do_fetch_actresses();
                } else {
                    app.do_fetch_next_page();
                }
            }
        }
        KeyCode::Char('p') => {
            if app.page > 1 {
                app.page -= 1;
                app.loading = true;
                if app.tab == Tab::Actresses {
                    app.do_fetch_actresses();
                } else {
                    app.do_fetch_next_page();
                }
            }
        }

        // page jump
        KeyCode::Char('N') => {
            app.show_page_jump = true;
            app.page_jump_input.clear();
        }

        // toggle logs
        KeyCode::Char('~') => { app.show_logs = !app.show_logs; }

        _ => {}
    }
    false
}

pub fn handle_click(app: &mut App, col: u16, row: u16) {
    let (width, _) = crossterm::terminal::size().unwrap_or((80, 24));
    let left_width = width * 45 / 100;

    // tab row click (row 0 = border with tab labels)
    if row == 0 && col < left_width {
        // tabs: "[🔍影片] │ 👩女优 │ 📋历史 │ ⭐收藏"
        // approximate positions based on character widths
        let positions = [
            (1, 10, Tab::Movies),     // 🔍影片
            (12, 20, Tab::Actresses),  // 👩女优
            (22, 30, Tab::History),    // 📋历史
            (32, 42, Tab::Favorites),  // ⭐收藏
        ];
        for (start, end, tab) in positions {
            if col >= start && col < end {
                let old = app.tab;
                app.tab = tab;
                if old != tab {
                    app.panel = Panel::Left;
                    match tab {
                        Tab::History => app.load_history(),
                        Tab::Favorites => app.load_favorites(),
                        _ => {}
                    }
                }
                return;
            }
        }
    }

    // left panel list click → move cursor to clicked item
    if col < left_width && row > 1 {
        app.panel = Panel::Left;
        let content_row = row.saturating_sub(2); // border + info bar
        let card_h = if app.config.show_thumbnails { 10u16 } else { 3u16 };
        let grid_cols = app.grid_cols() as u16;
        let col_w = left_width / grid_cols;
        let grid_row = content_row / card_h;
        let grid_col = col / col_w;

        // calculate scroll offset from current cursor
        let cur = app.cursor();
        let rows_per_page = (area_height_approx(row) / card_h).max(1) as usize;
        let cursor_row = cur / grid_cols as usize;
        let scroll_row = if cursor_row >= rows_per_page { cursor_row - rows_per_page + 1 } else { 0 };

        let clicked_idx = (scroll_row + grid_row as usize) * grid_cols as usize + grid_col as usize;
        let max = if app.tab == Tab::Actresses { app.actresses.len() } else { app.current_movies().len() };

        if clicked_idx < max {
            *app.cursor_mut() = clicked_idx;
            app.select_current();
        }
        return;
    }

    // right panel click
    app.panel = if col < left_width { Panel::Left } else { Panel::Right };
}

fn area_height_approx(_click_row: u16) -> u16 {
    let (_, h) = crossterm::terminal::size().unwrap_or((80, 40));
    h.saturating_sub(10) // approximate list area height (minus borders, info, logs, status)
}

fn load_tab_data(app: &mut App) {
    match app.tab {
        Tab::History => app.load_history(),
        Tab::Favorites => app.load_favorites(),
        Tab::Movies | Tab::Actresses => {} // data already separate, no action needed
    }
}
