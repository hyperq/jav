use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, event::EnableMouseCapture, event::DisableMouseCapture};
use image::DynamicImage;
use ratatui::prelude::*;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::picker::{Picker, ProtocolType};
use tokio::sync::mpsc;

use crate::scraper::{Actress, JavClient, Magnet, Movie, MovieDetail, PageResult};
use crate::store::Store;
use crate::cloud115::{self, Client115, QrLogin, QrStatus, SavedCookie};

use super::render;
use super::input;

// --- State types ---

#[derive(Clone, Copy, PartialEq)]
pub enum Panel { Left, Right }

#[derive(Clone, Copy, PartialEq)]
pub enum Tab { Movies, Actresses, History, Favorites }

#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode { Movie, Actress, StarCode }

pub enum AsyncMsg {
    PageResult(Result<PageResult>),
    Magnets(String, Result<Vec<Magnet>>),
    ActressList(Result<Vec<Actress>>),
    MovieDetailResult(String, Result<MovieDetail>),
    ImageLoaded(String, Option<DynamicImage>),
    ExportProgress(usize, usize), // (done, total)
    ExportDone(usize, String),
    // 115 cloud
    Cloud115QrImage(Option<DynamicImage>), // downloaded QR image
    Cloud115QrStatus(QrStatus),
    Cloud115LoginDone(Result<SavedCookie>),
    Cloud115LoginCheck(bool),
    Cloud115Quota(cloud115::CloudQuota),
    Cloud115TaskResult(Result<String>),
    Cloud115BatchProgress(usize, usize), // (done, total) fetching magnets
    Cloud115BatchResult(Result<cloud115::BatchAddResult>),
}

// --- Config ---

#[derive(Clone)]
pub struct AppConfig {
    pub show_thumbnails: bool,
    pub show_cover: bool,       // 详情页封面
    pub prefer_caption: bool,
    pub prefer_hd: bool,
    pub sort_by_size: bool,
    pub kitty_supported: bool,  // runtime: terminal supports Kitty graphics
}

impl AppConfig {
    pub fn load(store: &Store) -> Self {
        Self {
            show_thumbnails: store.get_config_bool("show_thumbnails", true),
            show_cover: store.get_config_bool("show_cover", true),
            prefer_caption: store.get_config_bool("prefer_caption", true),
            prefer_hd: store.get_config_bool("prefer_hd", true),
            sort_by_size: store.get_config_bool("sort_by_size", true),
            kitty_supported: false, // set at runtime
        }
    }

    pub fn save(&self, store: &Store) {
        store.set_config_bool("show_thumbnails", self.show_thumbnails);
        store.set_config_bool("show_cover", self.show_cover);
        store.set_config_bool("prefer_caption", self.prefer_caption);
        store.set_config_bool("prefer_hd", self.prefer_hd);
        store.set_config_bool("sort_by_size", self.sort_by_size);
    }

    pub fn images_enabled(&self) -> bool {
        self.kitty_supported && (self.show_thumbnails || self.show_cover)
    }
}

// --- App ---

pub struct App {
    pub client: Arc<JavClient>,
    pub store: Store,
    pub tx: mpsc::UnboundedSender<AsyncMsg>,
    pub rx: mpsc::UnboundedReceiver<AsyncMsg>,

    pub panel: Panel,
    pub tab: Tab,

    // search popup
    pub show_search: bool,
    pub search_mode: SearchMode,
    pub search_input: String,
    pub uncensored: bool,
    pub show_page_jump: bool,
    pub page_jump_input: String,

    // per-tab data (independent lists)
    pub search_movies: Vec<Movie>,    // 影片 tab
    pub search_cursor: usize,
    pub search_info: String,

    pub actresses: Vec<Actress>,      // 女优 tab
    pub actress_cursor: usize,

    pub history_movies: Vec<Movie>,   // 历史 tab
    pub history_cursor: usize,

    pub favorite_movies: Vec<Movie>,  // 收藏 tab
    pub favorite_cursor: usize,

    pub last_search_mode: SearchMode,
    pub page: usize,
    pub has_next: bool,
    pub keyword: String,
    pub loading: bool,
    pub error: Option<String>,

    // right panel: detail
    pub selected_movie: Option<Movie>,
    pub movie_detail: Option<MovieDetail>,
    pub magnets: Vec<Magnet>,
    pub right_scroll: u16,

    // multi-select & export (by movie number, persists across searches)
    pub selected_numbers: std::collections::HashSet<String>,
    pub selected_movies_cache: Vec<Movie>, // cached full Movie data for export
    pub export_progress: Option<(usize, usize)>,

    // config
    pub show_config: bool,
    pub config: AppConfig,

    // images
    pub picker: Picker,
    pub image_cache: HashMap<String, StatefulProtocol>,
    pub image_loading: std::collections::HashSet<String>,

    // 115 cloud
    pub cloud115: Option<Client115>,
    pub cloud115_checked: bool,
    pub cloud115_quota: Option<cloud115::CloudQuota>,
    pub show_qr_login: bool,
    pub qr_image_key: Option<String>, // key into image_cache for QR image
    pub qr_status_text: String,
    pub show_download_input: bool,
    pub download_dir: String,
    pub pending_magnets: Vec<String>,
    pub pending_batch_movies: Vec<Movie>,
    pub cloud_dl_progress: Option<(usize, usize)>, // batch progress (done, total)

    // ui state
    pub logs: Vec<String>,
    pub show_logs: bool,
    pub toast: Option<(String, std::time::Instant)>,
}

impl App {
    pub fn new(client: JavClient, store: Store) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        // try to query terminal for font size, fallback to env or default
        let mut picker = Picker::from_query_stdio()
            .unwrap_or_else(|_| {
                // allow override via JAV_FONT_SIZE="w,h" env
                let fs = std::env::var("JAV_FONT_SIZE").ok()
                    .and_then(|s| {
                        let parts: Vec<&str> = s.split(',').collect();
                        if parts.len() == 2 {
                            Some((parts[0].trim().parse::<u16>().ok()?, parts[1].trim().parse::<u16>().ok()?))
                        } else { None }
                    })
                    .unwrap_or((8, 16));
                Picker::from_fontsize(fs)
            });
        let detected_proto = picker.protocol_type();
        let detected_font = picker.font_size();

        // check if terminal supports Kitty graphics protocol
        let kitty_supported = matches!(detected_proto, ProtocolType::Kitty)
            || std::env::var("TERM").ok().map(|t| t.contains("kitty") || t.contains("xterm-kitty")).unwrap_or(false)
            || std::env::var("TERM_PROGRAM").ok().map(|t| t.contains("ghostty") || t.contains("kitty") || t.contains("WezTerm")).unwrap_or(false);

        if kitty_supported {
            picker.set_protocol_type(ProtocolType::Kitty);
        }

        let init_log = format!(
            "🖼️ Picker: detected={:?} font={:?} kitty={} | 若图太小用 JAV_FONT_SIZE=w,h",
            detected_proto, detected_font, kitty_supported
        );

        let mut config = AppConfig::load(&store);
        config.kitty_supported = kitty_supported;
        if !kitty_supported {
            config.show_thumbnails = false;
            config.show_cover = false;
        }

        let mut app = Self {
            client: Arc::new(client),
            store,
            tx, rx,
            panel: Panel::Left,
            tab: Tab::History,
            show_search: false,
            search_mode: SearchMode::Movie,
            search_input: String::new(),
            uncensored: false,
            show_page_jump: false,
            page_jump_input: String::new(),
            search_movies: vec![],
            search_cursor: 0,
            search_info: String::new(),
            actresses: vec![],
            actress_cursor: 0,
            history_movies: vec![],
            history_cursor: 0,
            favorite_movies: vec![],
            favorite_cursor: 0,
            last_search_mode: SearchMode::Movie,
            page: 1,
            has_next: false,
            keyword: String::new(),
            loading: false,
            error: None,
            selected_movie: None,
            movie_detail: None,
            magnets: vec![],
            right_scroll: 0,
            selected_numbers: std::collections::HashSet::new(),
            selected_movies_cache: vec![],
            export_progress: None,
            show_config: false,
            config,
            picker,
            image_cache: HashMap::new(),
            image_loading: std::collections::HashSet::new(),
            cloud115: None,
            cloud115_checked: false,
            cloud115_quota: None,
            show_qr_login: false,
            qr_image_key: None,
            qr_status_text: String::new(),
            show_download_input: false,
            download_dir: String::new(),
            pending_magnets: vec![],
            pending_batch_movies: vec![],
            cloud_dl_progress: None,
            logs: vec![],
            show_logs: false,
            toast: None,
        };

        // try loading saved 115 cookie + async validation
        if let Some(cookie) = cloud115::load_cookie() {
            let cookie_clone = cookie.clone();
            match Client115::new(cookie) {
                Ok(c) => {
                    app.cloud115 = Some(c);
                    app.log("☁️ 115 cookie 已加载，验证中...".into());
                    let tx = app.tx.clone();
                    tokio::spawn(async move {
                        let valid = cloud115::check_login(&cookie_clone).await;
                        let _ = tx.send(AsyncMsg::Cloud115LoginCheck(valid));
                    });
                }
                Err(e) => app.log(format!("⚠️ 115 cookie 加载失败: {e}")),
            }
        }

        app.log("🚀 JAV TUI 启动".into());
        app.log(init_log);

        app.load_history();
        app.load_favorites();

        app
    }

    pub fn show_toast(&mut self, msg: &str) {
        self.toast = Some((msg.to_string(), std::time::Instant::now()));
        self.log(msg.to_string());
    }

    pub fn grid_cols(&self) -> usize {
        match self.tab {
            Tab::Actresses => 3,
            _ => 2,
        }
    }

    pub fn current_movies(&self) -> &[Movie] {
        match self.tab {
            Tab::Movies => &self.search_movies,
            Tab::History => &self.history_movies,
            Tab::Favorites => &self.favorite_movies,
            Tab::Actresses => &[], // actresses use separate list
        }
    }

    pub fn current_movies_mut(&mut self) -> &mut Vec<Movie> {
        match self.tab {
            Tab::Movies => &mut self.search_movies,
            Tab::History => &mut self.history_movies,
            Tab::Favorites => &mut self.favorite_movies,
            Tab::Actresses => &mut self.search_movies, // fallback
        }
    }

    pub fn cursor(&self) -> usize {
        match self.tab {
            Tab::Movies => self.search_cursor,
            Tab::Actresses => self.actress_cursor,
            Tab::History => self.history_cursor,
            Tab::Favorites => self.favorite_cursor,
        }
    }

    pub fn cursor_mut(&mut self) -> &mut usize {
        match self.tab {
            Tab::Movies => &mut self.search_cursor,
            Tab::Actresses => &mut self.actress_cursor,
            Tab::History => &mut self.history_cursor,
            Tab::Favorites => &mut self.favorite_cursor,
        }
    }

    pub fn load_history(&mut self) {
        if let Ok(items) = self.store.get_history(200) {
            self.history_movies = items.iter().map(|h| {
                let mut tags: Vec<String> = h.tags.split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                if h.favorited && !tags.iter().any(|t| t.contains("⭐")) {
                    tags.push("⭐".into());
                }
                Movie {
                    number: h.number.clone(),
                    title: h.title.clone(),
                    link: h.link.clone(),
                    cover: h.cover.clone(),
                    date: h.viewed_at.chars().take(10).collect(),
                    tags,
                }
            }).collect();
        }
    }

    pub fn load_favorites(&mut self) {
        if let Ok(items) = self.store.get_favorites(200) {
            self.favorite_movies = items.iter().map(|h| {
                let mut tags: Vec<String> = h.tags.split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                if !tags.iter().any(|t| t.contains("⭐")) {
                    tags.push("⭐".into());
                }
                Movie {
                    number: h.number.clone(),
                    title: h.title.clone(),
                    link: h.link.clone(),
                    cover: h.cover.clone(),
                    date: h.viewed_at.chars().take(10).collect(),
                    tags,
                }
            }).collect();
        }
    }

    pub fn log(&mut self, msg: String) {
        if self.logs.len() > 100 { self.logs.remove(0); }
        self.logs.push(msg.clone());
        // also write to file for debugging
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/jav-tui.log") {
            let _ = writeln!(f, "{msg}");
        }
    }

    // --- Search ---

    pub fn open_search(&mut self, mode: SearchMode) {
        self.show_search = true;
        self.search_mode = mode;
        self.search_input.clear();
    }

    pub fn execute_search(&mut self) {
        if self.search_input.is_empty() { return; }
        self.keyword = self.search_input.clone();
        self.show_search = false;
        self.page = 1;
        *self.cursor_mut() = 0;
        self.loading = true;
        self.error = None;
        self.last_search_mode = self.search_mode;

        let label = if self.uncensored { "无码" } else { "有码" };
        match self.search_mode {
            SearchMode::Movie => {
                self.tab = Tab::Movies;
                self.log(format!("🔍 搜索影片[{label}]: {}", self.keyword));
                self.do_fetch_page();
            }
            SearchMode::Actress => {
                self.tab = Tab::Actresses;
                self.log(format!("👩 搜索女优[{label}]: {}", self.keyword));
                self.do_fetch_actresses();
            }
            SearchMode::StarCode => {
                self.tab = Tab::Movies;
                self.log(format!("👩 女优代号: {}", self.keyword));
                self.do_fetch_star_movies();
            }
        }
    }

    // --- Async operations ---

    pub fn do_fetch_next_page(&self) {
        match self.last_search_mode {
            SearchMode::StarCode => self.do_fetch_star_movies(),
            SearchMode::Movie => self.do_fetch_page(),
            SearchMode::Actress => self.do_fetch_page(),
        }
    }

    pub fn do_fetch_page(&self) {
        let client = self.client.clone();
        let kw = self.keyword.clone();
        let page = self.page;
        let uc = self.uncensored;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.fetch_page_ex(&kw, page, uc).await;
            let _ = tx.send(AsyncMsg::PageResult(r));
        });
    }

    pub fn do_fetch_actresses(&self) {
        let client = self.client.clone();
        let kw = self.keyword.clone();
        let uc = self.uncensored;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.fetch_actresses(&kw, 1, uc).await;
            let _ = tx.send(AsyncMsg::ActressList(r.map(|p| p.actresses)));
        });
    }

    pub fn do_fetch_star_movies(&self) {
        let client = self.client.clone();
        let code = self.keyword.clone();
        let page = self.page;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.fetch_star_movies(&code, page).await;
            let _ = tx.send(AsyncMsg::PageResult(r));
        });
    }

    pub fn do_fetch_magnets(&self, movie: &Movie) {
        let client = self.client.clone();
        let movie = movie.clone();
        let tx = self.tx.clone();
        let number = movie.number.clone();
        tokio::spawn(async move {
            let r = client.fetch_magnets(&movie).await;
            let _ = tx.send(AsyncMsg::Magnets(number, r));
        });
    }

    pub fn do_fetch_detail(&self, movie: &Movie) {
        let client = self.client.clone();
        let url = movie.link.clone();
        let number = movie.number.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.fetch_movie_detail(&url).await;
            let _ = tx.send(AsyncMsg::MovieDetailResult(number, r));
        });
    }

    pub fn do_fetch_image(&mut self, url: &str, key: &str) {
        if !self.config.kitty_supported || url.is_empty() || self.image_cache.contains_key(key) || self.image_loading.contains(key) {
            return;
        }
        self.image_loading.insert(key.to_string());
        let client = self.client.clone();
        let url = url.to_string();
        let key = key.to_string();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            // try disk cache first
            let cache_path = image_cache_path(&key);
            let img = if let Ok(bytes) = tokio::fs::read(&cache_path).await {
                image::load_from_memory(&bytes).ok()
            } else {
                // fetch from network and save to cache
                match client.fetch_image_bytes(&url).await {
                    Ok(bytes) => {
                        // save to disk cache
                        if let Some(dir) = std::path::Path::new(&cache_path).parent() {
                            let _ = tokio::fs::create_dir_all(dir).await;
                        }
                        let _ = tokio::fs::write(&cache_path, &bytes).await;
                        image::load_from_memory(&bytes).ok()
                    }
                    Err(_) => None,
                }
            };
            let _ = tx.send(AsyncMsg::ImageLoaded(key, img));
        });
    }

    pub fn clear_image_cache(&mut self) {
        let cache_dir = image_cache_dir();
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            let count = entries.filter_map(|e| e.ok()).filter(|e| std::fs::remove_file(e.path()).is_ok()).count();
            self.log(format!("🗑️ 已清理 {} 个缓存图片", count));
        }
        self.image_cache.clear();
    }

    // --- Select item ---

    pub fn select_current(&mut self) {
        match self.tab {
            Tab::Movies | Tab::History | Tab::Favorites => {
                let cursor = self.cursor();
                if let Some(m) = self.current_movies().get(cursor).cloned() {
                    self.log(format!("📄 {}", m.number));
                    let tags_str = m.tags.join(",");
                    let _ = self.store.add_history(&m.number, &m.title, &m.link, &m.cover, &tags_str, "");
                    // remove old cover cache so detail page gets fresh big cover
                    let old_key = format!("cover_{}", m.number);
                    self.image_cache.remove(&old_key);
                    self.image_loading.remove(&old_key);
                    self.selected_movie = Some(m.clone());
                    self.movie_detail = None;
                    self.magnets.clear();
                    self.right_scroll = 0;
                    self.panel = Panel::Right;
                    self.do_fetch_detail(&m);
                    self.do_fetch_magnets(&m);
                }
            }
            Tab::Actresses => {
                if let Some(a) = self.actresses.get(self.actress_cursor).cloned() {
                    self.log(format!("👩 {} → 作品列表", a.name));
                    self.keyword = a.code.clone();
                    self.last_search_mode = SearchMode::StarCode;
                    self.tab = Tab::Movies;
                    self.page = 1;
                    self.search_cursor = 0;
                    self.loading = true;
                    self.do_fetch_star_movies();
                }
            }
        }
    }

    pub fn toggle_select(&mut self) {
        let cursor = self.cursor();
        let movie = self.current_movies().get(cursor).cloned();
        if let Some(m) = movie {
            let num = m.number.clone();
            if self.selected_numbers.contains(&num) {
                self.selected_numbers.remove(&num);
                self.selected_movies_cache.retain(|c| c.number != num);
            } else {
                self.selected_numbers.insert(num);
                self.selected_movies_cache.push(m);
            }
        }
    }

    pub fn select_all(&mut self) {
        let movies: Vec<Movie> = self.current_movies().to_vec();
        let all_selected = movies.iter().all(|m| self.selected_numbers.contains(&m.number));
        if all_selected {
            for m in &movies {
                self.selected_numbers.remove(&m.number);
            }
            let nums: std::collections::HashSet<String> = movies.iter().map(|m| m.number.clone()).collect();
            self.selected_movies_cache.retain(|c| !nums.contains(&c.number));
        } else {
            for m in &movies {
                if !self.selected_numbers.contains(&m.number) {
                    self.selected_numbers.insert(m.number.clone());
                    self.selected_movies_cache.push(m.clone());
                }
            }
        }
    }

    pub fn toggle_favorite(&mut self) {
        let movie = if self.panel == Panel::Left {
            let cursor = self.cursor();
            self.current_movies().get(cursor).cloned()
        } else {
            self.selected_movie.clone()
        };

        if let Some(m) = movie {
            // ensure it's in history first
            let tags_str = m.tags.join(",");
                    let _ = self.store.add_history(&m.number, &m.title, &m.link, &m.cover, &tags_str, "");
            // toggle favorite
            if let Ok(items) = self.store.get_history(200) {
                if let Some(item) = items.iter().find(|i| i.number == m.number) {
                    let _ = self.store.toggle_favorite(item.id);
                    let status = if item.favorited { "取消收藏" } else { "已收藏" };
                    self.log(format!("⭐ {}: {}", status, m.number));
                }
            }
        }
    }

    pub fn export_selected_magnets(&mut self) {
        if self.selected_numbers.is_empty() {
            self.show_toast("⚠️ 未选择任何影片，先按空格选择");
            return;
        }

        let movies: Vec<Movie> = self.selected_movies_cache.clone();
        let client = self.client.clone();
        let config = self.config.clone();
        let tx = self.tx.clone();

        let total = movies.len();
        self.export_progress = Some((0, total));
        self.log(format!("📦 批量获取 {} 部影片磁链...", total));

        tokio::spawn(async move {
            let mut results = Vec::new();
            for (i, movie) in movies.iter().enumerate() {
                let _ = tx.send(AsyncMsg::ExportProgress(i + 1, total));
                match client.fetch_magnets(movie).await {
                    Ok(mut mags) => {
                        // sort by config
                        mags.sort_by(|a, b| {
                            let mut ord = std::cmp::Ordering::Equal;
                            if config.prefer_caption {
                                ord = ord.then(b.caption.cmp(&a.caption));
                            }
                            if config.prefer_hd {
                                ord = ord.then(b.size.contains("HD").cmp(&a.size.contains("HD")));
                            }
                            if config.sort_by_size {
                                let sa = parse_size_mb(&a.size);
                                let sb = parse_size_mb(&b.size);
                                ord = ord.then(sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal));
                            }
                            ord
                        });
                        if let Some(best) = mags.first() {
                            results.push(best.link.clone());
                        }
                    }
                    Err(_) => {}
                }
            }

            // write to file in current directory
            let cwd = std::env::current_dir().unwrap_or_default();
            let filename = cwd.join("magnets_export.txt");
            let content = results.join("\n");
            let _ = std::fs::write(&filename, &content);
            let path_str = filename.to_string_lossy().to_string();
            let _ = tx.send(AsyncMsg::ExportDone(results.len(), path_str));
        });
    }

    /// get magnet for current selected movie (smart sort: caption > hd > size)
    pub fn grab_magnet(&mut self) {
        if self.magnets.is_empty() {
            let cursor = self.cursor();
            if let Some(m) = self.current_movies().get(cursor).cloned() {
                self.log(format!("🧲 获取磁链: {}", m.number));
                self.do_fetch_magnets(&m);
            }
            return;
        }
        let best = self.pick_best_magnet();
        if let Some(mag) = best {
            self.log(format!("✅ 最佳磁链: {} [{}]", if mag.caption {"字幕"} else {""}, mag.size));
            // TODO: copy to clipboard
        }
    }

    fn pick_best_magnet(&self) -> Option<&Magnet> {
        if self.magnets.is_empty() { return None; }
        // sort logic: caption first, then by size descending
        let mut sorted: Vec<&Magnet> = self.magnets.iter().collect();
        sorted.sort_by(|a, b| {
            b.caption.cmp(&a.caption)
                .then_with(|| {
                    let sa = parse_size_mb(&a.size);
                    let sb = parse_size_mb(&b.size);
                    sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        sorted.first().copied()
    }

    // --- 115 cloud ---

    pub fn fetch_115_quota(&self) {
        let cookie = match &self.cloud115 {
            Some(c) => c.cookie().clone(),
            None => return,
        };
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Ok(client) = Client115::new(cookie) {
                if let Ok(quota) = client.get_quota().await {
                    let _ = tx.send(AsyncMsg::Cloud115Quota(quota));
                }
            }
        });
    }

    pub fn start_qr_login(&mut self) {
        self.show_qr_login = true;
        self.qr_image_key = None;
        self.image_cache.remove("qr_115");
        self.qr_status_text = "正在获取二维码...".into();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match QrLogin::new() {
                Ok(mut login) => {
                    match login.request_token().await {
                        Ok(_) => {
                            // download QR image from 115
                            match login.fetch_qr_image().await {
                                Ok(bytes) => {
                                    let img = image::load_from_memory(&bytes).ok();
                                    let _ = tx.send(AsyncMsg::Cloud115QrImage(img));
                                }
                                Err(_) => {
                                    let _ = tx.send(AsyncMsg::Cloud115QrImage(None));
                                }
                            }
                            // poll loop
                            loop {
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                match login.poll_status().await {
                                    Ok(status) => {
                                        let done = matches!(status, QrStatus::Confirmed | QrStatus::Expired | QrStatus::Canceled);
                                        let _ = tx.send(AsyncMsg::Cloud115QrStatus(status.clone()));
                                        if status == QrStatus::Confirmed {
                                            match login.finish_login().await {
                                                Ok(cookie) => { let _ = tx.send(AsyncMsg::Cloud115LoginDone(Ok(cookie))); }
                                                Err(e) => { let _ = tx.send(AsyncMsg::Cloud115LoginDone(Err(e))); }
                                            }
                                            break;
                                        }
                                        if done { break; }
                                    }
                                    Err(e) => {
                                        let _ = tx.send(AsyncMsg::Cloud115LoginDone(Err(e)));
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => { let _ = tx.send(AsyncMsg::Cloud115LoginDone(Err(e))); }
                    }
                }
                Err(e) => { let _ = tx.send(AsyncMsg::Cloud115LoginDone(Err(e))); }
            }
        });
    }

    // d: download detail page's best magnet
    pub fn download_detail_to_115(&mut self) {
        if self.cloud115.is_none() {
            self.show_toast("⚠️ 未登录 115，按 L 扫码登录");
            return;
        }
        let magnet = self.pick_best_magnet().map(|m| m.link.clone());
        if let Some(link) = magnet {
            self.pending_magnets = vec![link];
            self.show_download_input = true;
            self.download_dir.clear();
        } else {
            self.show_toast("⚠️ 当前无可用磁力链接");
        }
    }

    // D: batch download selected movies (fetches magnets for each)
    pub fn download_selected_to_115(&mut self) {
        if self.cloud115.is_none() {
            self.show_toast("⚠️ 未登录 115，按 L 扫码登录");
            return;
        }
        if self.selected_numbers.is_empty() {
            self.show_toast("⚠️ 未选择影片，先按空格选择");
            return;
        }
        // collect selected movies, then show dir input
        self.pending_batch_movies = self.selected_movies_cache.clone();
        self.pending_magnets = vec![];
        self.show_download_input = true;
        self.download_dir.clear();
        self.log(format!("☁️ 准备批量下载 {} 部到 115", self.pending_batch_movies.len()));
    }

    pub fn confirm_download(&mut self) {
        self.show_download_input = false;
        let dir_id = self.download_dir.clone();
        let cookie = self.cloud115.as_ref().unwrap().cookie().clone();
        let tx = self.tx.clone();

        if !self.pending_magnets.is_empty() {
            // single mode (d): one magnet, one call
            let magnet = std::mem::take(&mut self.pending_magnets);
            let number = self.selected_movie.as_ref().map(|m| m.number.clone()).unwrap_or_default();
            self.log(format!("☁️ 提交到 115: {number}"));
            tokio::spawn(async move {
                match Client115::new(cookie) {
                    Ok(client) => {
                        let result = client.add_task(&magnet[0], &dir_id).await;
                        let _ = tx.send(AsyncMsg::Cloud115TaskResult(result));
                    }
                    Err(e) => { let _ = tx.send(AsyncMsg::Cloud115TaskResult(Err(e))); }
                }
            });
        } else if !self.pending_batch_movies.is_empty() {
            // batch mode (D): fetch magnets → collect URLs → one batch call
            let movies = std::mem::take(&mut self.pending_batch_movies);
            let jav_client = self.client.clone();
            let config = self.config.clone();
            let total = movies.len();
            self.cloud_dl_progress = Some((0, total));
            self.log(format!("☁️ 批量获取 {} 部磁链中...", total));
            tokio::spawn(async move {
                let mut urls: Vec<String> = Vec::new();
                for (i, movie) in movies.iter().enumerate() {
                    let _ = tx.send(AsyncMsg::Cloud115BatchProgress(i + 1, total));
                    if let Ok(mut mags) = jav_client.fetch_magnets(movie).await {
                        mags.sort_by(|a, b| {
                            let mut ord = std::cmp::Ordering::Equal;
                            if config.prefer_caption { ord = ord.then(b.caption.cmp(&a.caption)); }
                            if config.sort_by_size {
                                let sa = parse_size_mb(&a.size);
                                let sb = parse_size_mb(&b.size);
                                ord = ord.then(sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal));
                            }
                            ord
                        });
                        if let Some(best) = mags.first() {
                            urls.push(best.link.clone());
                        }
                    }
                }
                if urls.is_empty() {
                    let _ = tx.send(AsyncMsg::Cloud115BatchResult(Err(anyhow::anyhow!("未找到可用磁链"))));
                    return;
                }
                match Client115::new(cookie) {
                    Ok(client) => {
                        let result = client.add_tasks(&urls, &dir_id).await;
                        let _ = tx.send(AsyncMsg::Cloud115BatchResult(result));
                    }
                    Err(e) => { let _ = tx.send(AsyncMsg::Cloud115BatchResult(Err(e))); }
                }
            });
        }
    }

    // --- Process async messages ---

    pub fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMsg::PageResult(Ok(r)) => {
                    self.log(format!("✅ 第{}页: {}条", r.page, r.movies.len()));
                    self.has_next = r.has_next;
                    if !r.result_info.is_empty() {
                        self.search_info = r.result_info;
                    }
                    if r.page == 1 {
                        self.search_movies = r.movies;
                        self.search_cursor = 0;
                    } else {
                        self.search_movies.extend(r.movies);
                    }
                    self.loading = false;
                }
                AsyncMsg::PageResult(Err(e)) => {
                    self.log(format!("❌ {e}"));
                    self.error = Some(e.to_string());
                    self.loading = false;
                }
                AsyncMsg::ActressList(Ok(list)) => {
                    self.log(format!("👩 {}位女优", list.len()));
                    self.has_next = list.len() >= 50;
                    if self.page == 1 {
                        self.actresses = list;
                        self.actress_cursor = 0;
                    } else {
                        self.actresses.extend(list);
                    }
                    self.loading = false;
                }
                AsyncMsg::ActressList(Err(e)) => {
                    self.log(format!("❌ {e}"));
                    self.error = Some(e.to_string());
                    self.loading = false;
                }
                AsyncMsg::Magnets(number, Ok(mags)) => {
                    self.log(format!("🧲 {number}: {}条磁链", mags.len()));
                    // cache magnets to db
                    let mag_str = mags.iter().map(|m| {
                        format!("{}|{}|{}", m.link, m.size, if m.caption {"1"} else {"0"})
                    }).collect::<Vec<_>>().join("\n");
                    let _ = self.store.update_magnets(&number, &mag_str);

                    if self.selected_movie.as_ref().is_some_and(|m| m.number == number) {
                        self.magnets = mags;
                    }
                }
                AsyncMsg::Magnets(_, Err(e)) => {
                    self.log(format!("❌ 磁链: {e}"));
                }
                AsyncMsg::MovieDetailResult(number, Ok(detail)) => {
                    self.log(format!("📄 {number} 详情OK"));
                    if self.selected_movie.as_ref().is_some_and(|m| m.number == number) {
                        // fetch the big cover from detail page
                        let cover_key = format!("cover_{}", number);
                        if !detail.cover.is_empty() {
                            self.image_loading.remove(&cover_key);
                            self.image_cache.remove(&cover_key);
                            self.do_fetch_image(&detail.cover, &cover_key);
                        }
                        self.movie_detail = Some(detail);
                    }
                }
                AsyncMsg::MovieDetailResult(_, Err(e)) => {
                    self.log(format!("❌ 详情: {e}"));
                }
                AsyncMsg::ExportProgress(done, total) => {
                    self.export_progress = Some((done, total));
                }
                AsyncMsg::ExportDone(count, path) => {
                    self.export_progress = None;
                    self.log(format!("✅ 导出完成: {}条磁链 → {}", count, path));
                    self.selected_numbers.clear();
                    self.selected_movies_cache.clear();
                }
                // 115 cloud messages
                AsyncMsg::Cloud115QrImage(Some(img)) => {
                    let proto = self.picker.new_resize_protocol(img);
                    self.image_cache.insert("qr_115".into(), proto);
                    self.qr_image_key = Some("qr_115".into());
                    self.qr_status_text = "请用 115 APP 扫码".into();
                }
                AsyncMsg::Cloud115QrImage(None) => {
                    self.qr_status_text = "⚠️ 二维码加载失败，按 r 重试".into();
                }
                AsyncMsg::Cloud115QrStatus(status) => {
                    self.qr_status_text = status.label().into();
                }
                AsyncMsg::Cloud115LoginCheck(valid) => {
                    self.cloud115_checked = true;
                    if valid {
                        self.log("☁️ 115 cookie 有效".into());
                        self.fetch_115_quota();
                    } else {
                        self.log("⚠️ 115 cookie 已过期，按 L 重新登录".into());
                        self.cloud115 = None;
                    }
                }
                AsyncMsg::Cloud115Quota(quota) => {
                    self.log(format!("☁️ 115 配额: 剩余{}/总共{}", quota.quota, quota.total));
                    self.cloud115_quota = Some(quota);
                }
                AsyncMsg::Cloud115LoginDone(Ok(cookie)) => {
                    self.log("☁️ 115 登录成功！".into());
                    let _ = cloud115::save_cookie(&cookie);
                    match Client115::new(cookie) {
                        Ok(c) => {
                            self.cloud115 = Some(c);
                            self.cloud115_checked = true;
                            self.fetch_115_quota();
                        }
                        Err(e) => { self.log(format!("⚠️ 115 client 创建失败: {e}")); }
                    }
                    self.show_qr_login = false;
                }
                AsyncMsg::Cloud115LoginDone(Err(e)) => {
                    self.log(format!("❌ 115 登录失败: {e}"));
                    self.qr_status_text = format!("登录失败: {e}");
                }
                AsyncMsg::Cloud115TaskResult(Ok(name)) => {
                    self.show_toast(&format!("✅ 已提交: {name}"));
                    self.log(format!("✅ 115 云下载已提交: {name}"));
                }
                AsyncMsg::Cloud115TaskResult(Err(e)) => {
                    self.show_toast(&format!("❌ 下载失败: {e}"));
                    self.log(format!("❌ 115 云下载失败: {e}"));
                }
                AsyncMsg::Cloud115BatchProgress(done, total) => {
                    self.cloud_dl_progress = Some((done, total));
                }
                AsyncMsg::Cloud115BatchResult(Ok(result)) => {
                    self.cloud_dl_progress = None;
                    let msg = if result.fail == 0 {
                        format!("✅ 批量提交成功: {}个任务", result.ok)
                    } else {
                        let errs = result.errors.first().cloned().unwrap_or_default();
                        format!("⚠️ 成功{}个 失败{}个: {}", result.ok, result.fail, errs)
                    };
                    self.show_toast(&msg);
                    self.log(msg);
                }
                AsyncMsg::Cloud115BatchResult(Err(e)) => {
                    self.cloud_dl_progress = None;
                    self.show_toast(&format!("❌ 批量下载失败: {e}"));
                    self.log(format!("❌ 115 批量下载失败: {e}"));
                }
                AsyncMsg::ImageLoaded(key, Some(img)) => {
                    self.log(format!("🖼️ loaded {}: {}x{}", key, img.width(), img.height()));
                    let font = self.picker.font_size();
                    // always resize to target pixel area (Retina needs explicit upscale)
                    let resized = if key.starts_with("actress_") {
                        // actress avatar: square, matching render area
                        // render uses: avatar_h = cell_h - 2 = 10 rows
                        // avatar_w = avatar_h * font_h / font_w (square in pixels)
                        let avatar_h_rows = 10u32;
                        let avatar_w_cols = avatar_h_rows * font.1 as u32 / font.0 as u32;
                        let target_w = avatar_w_cols * font.0 as u32;
                        let target_h = avatar_h_rows * font.1 as u32;
                        img.resize(target_w, target_h, image::imageops::FilterType::Lanczos3)
                    } else if key.starts_with("cover_") {
                        // detail cover: 100% width of right panel
                        let (term_w, _) = crossterm::terminal::size().unwrap_or((120, 40));
                        let cols = (term_w as u32 * 55 / 100).saturating_sub(2);
                        let target_w = cols * font.0 as u32;
                        // scale to exact width, height follows aspect ratio
                        let target_h = target_w * img.height() / img.width().max(1);
                        img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3)
                    } else {
                        // thumbnail: fit within 18 cols × 12 rows
                        let target_w = 18 * font.0 as u32;
                        let target_h = 12 * font.1 as u32;
                        img.resize(target_w, target_h, image::imageops::FilterType::Lanczos3)
                    };
                    self.log(format!("🖼️ resized {}: {}x{} → {}x{}",
                        key, img.width(), img.height(), resized.width(), resized.height()));
                    let proto = self.picker.new_resize_protocol(resized);
                    self.image_cache.insert(key.clone(), proto);
                    self.image_loading.remove(&key);
                }
                AsyncMsg::ImageLoaded(key, None) => {
                    self.image_loading.remove(&key);
                }
            }
        }
    }
}

fn image_cache_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{home}/.jav/cache")
}

fn image_cache_path(key: &str) -> String {
    let safe_key = key.replace('/', "_").replace(':', "_");
    format!("{}/{safe_key}.bin", image_cache_dir())
}

fn parse_size_mb(s: &str) -> f64 {
    let s = s.trim();
    let num: f64 = s.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect::<String>().parse().unwrap_or(0.0);
    if s.contains('G') { num * 1024.0 } else { num }
}

// --- Main loop ---

pub async fn run(client: JavClient, store: Store) -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client, store);

    loop {
        app.process_messages();
        // auto-clear expired toast
        if let Some((_, instant)) = &app.toast {
            if instant.elapsed() >= std::time::Duration::from_secs(3) {
                app.toast = None;
            }
        }
        terminal.draw(|f| render::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if input::handle_key(&mut app, key.code) {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            input::handle_click(&mut app, mouse.column, mouse.row);
                        }
                        MouseEventKind::ScrollUp => {
                            if app.panel == Panel::Left {
                                let step = app.grid_cols();
                                let cur = app.cursor();
                                *app.cursor_mut() = cur.saturating_sub(step);
                            } else {
                                app.right_scroll = app.right_scroll.saturating_sub(2);
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if app.panel == Panel::Left {
                                let max = if app.tab == Tab::Actresses { app.actresses.len() } else { app.current_movies().len() };
                                let step = app.grid_cols();
                                let cur = app.cursor();
                                if cur + step < max { *app.cursor_mut() = cur + step; }
                            } else {
                                app.right_scroll += 2;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
