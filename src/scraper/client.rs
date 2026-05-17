use anyhow::Result;
use reqwest::header::{REFERER, USER_AGENT};
use reqwest::redirect::Policy;
use scraper::{Html, Selector};

use super::types::{Actress, ActressDetail, ActressPageResult, Magnet, Movie, MovieDetail, PageResult};

pub struct JavClient {
    client: reqwest::Client,
    base_url: String,
}

impl JavClient {
    pub fn new(base_url: &str, proxy: Option<&str>) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .redirect(Policy::none())
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(REFERER, "https://www.javbus.com/".parse()?);
                h.insert(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".parse()?);
                h
            });

        if let Some(p) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }

        Ok(Self {
            client: builder.build()?,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn fetch_page(&self, keyword: &str, page: usize) -> Result<PageResult> {
        let url = self.build_list_url(keyword, page);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;

        let doc = Html::parse_document(&body);
        let box_sel = Selector::parse(".movie-box").unwrap();
        let date_sel = Selector::parse("date").unwrap();
        let img_sel = Selector::parse("img").unwrap();
        let tag_sel = Selector::parse(".item-tag button").unwrap();
        let _title_sel = Selector::parse(".photo-info span").unwrap();

        let mut movies = Vec::new();

        for el in doc.select(&box_sel) {
            let link = el.value().attr("href").unwrap_or_default().to_string();

            let dates: Vec<String> = el.select(&date_sel).map(|d| d.text().collect()).collect();
            let number = dates.first().cloned().unwrap_or_default();
            let date = dates.get(1).cloned().unwrap_or_default();

            let title = el
                .select(&img_sel)
                .next()
                .and_then(|i| i.value().attr("title"))
                .unwrap_or_default()
                .to_string();

            let cover = el
                .select(&img_sel)
                .next()
                .and_then(|i| i.value().attr("src"))
                .unwrap_or_default()
                .to_string();

            let tags: Vec<String> = el
                .select(&tag_sel)
                .map(|t| t.text().collect::<String>().trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();

            movies.push(Movie { number, title, link, cover, date, tags });
        }

        let result_info = Self::parse_result_info(&doc);
        let has_next = movies.len() >= 30;
        Ok(PageResult { movies, page, has_next, result_info })
    }

    pub async fn fetch_magnets(&self, movie: &Movie) -> Result<Vec<Magnet>> {
        let resp = self.client.get(&movie.link).send().await?;
        let body = resp.text().await?;

        let (gid, uc, img_val) = Self::parse_script_vars(&body);
        if gid.is_empty() {
            return Ok(vec![]);
        }

        // URL: /ajax/uncledatoolsbyajax.php?gid=XXX&lang=zh&img=YYY&uc=ZZZ&floor=NNN
        let ajax_url = format!(
            "{}/ajax/uncledatoolsbyajax.php?{}&lang=zh&{}&{}&floor={}",
            self.base_url, gid, img_val, uc, rand::random::<u16>() % 1000
        );

        let ajax_resp = self.client
            .get(&ajax_url)
            .header(REFERER, &movie.link)
            .send()
            .await?;
        let ajax_body = ajax_resp.text().await?;

        let wrapped = format!("<html><body><table>{}</table></body></html>", ajax_body);
        let doc = Html::parse_document(&wrapped);
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td").unwrap();
        let a_sel = Selector::parse("a[href^=\"magnet\"]").unwrap();

        let mut magnets = Vec::new();
        for tr in doc.select(&tr_sel) {
            let tds: Vec<_> = tr.select(&td_sel).collect();
            if tds.len() < 3 {
                continue;
            }

            // first td: magnet link + possible tags (高清/字幕)
            let link = tds[0]
                .select(&a_sel)
                .next()
                .and_then(|a| a.value().attr("href"))
                .unwrap_or_default()
                .to_string();

            if link.is_empty() || !link.starts_with("magnet:") {
                continue;
            }

            let td0_html = tds[0].inner_html();
            let caption = td0_html.contains("字幕");
            let hd = td0_html.contains("高清");

            // second td: size
            let size = tds[1].text().collect::<String>().trim().to_string();

            // add HD tag to size for display
            let size_display = if hd { format!("{} HD", size) } else { size };

            magnets.push(Magnet { link, size: size_display, caption });
        }

        Ok(magnets)
    }

    pub async fn fetch_image_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let full_url = if url.starts_with("http") {
            url.to_string()
        } else {
            format!("{}/{}", self.base_url, url.trim_start_matches('/'))
        };
        let bytes = self.client.get(&full_url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn fetch_actresses(&self, keyword: &str, page: usize, uncensored: bool) -> Result<ActressPageResult> {
        let url = self.build_actress_url(keyword, page, uncensored);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;

        let doc = Html::parse_document(&body);
        let box_sel = Selector::parse(".avatar-box").unwrap();
        let img_sel = Selector::parse("img").unwrap();
        let span_sel = Selector::parse("span").unwrap();

        let mut actresses = Vec::new();
        for el in doc.select(&box_sel) {
            let link = el.value().attr("href").unwrap_or_default().to_string();
            let code = link.rsplit('/').next().unwrap_or_default().to_string();
            let name = el.select(&span_sel).next().map(|s| s.text().collect::<String>()).unwrap_or_default();
            let avatar = el.select(&img_sel).next().and_then(|i| i.value().attr("src")).unwrap_or_default().to_string();

            actresses.push(Actress { code, name, avatar, link });
        }

        let has_next = actresses.len() >= 50;
        Ok(ActressPageResult { actresses, page, has_next })
    }

    pub async fn fetch_actress_detail(&self, code: &str) -> Result<(ActressDetail, PageResult)> {
        let url = format!("{}/star/{}", self.base_url, code);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;

        let doc = Html::parse_document(&body);

        // parse actress info
        let img_sel = Selector::parse(".photo-frame img, .avatar-box img").unwrap();
        let info_sel = Selector::parse(".photo-info p").unwrap();
        let avatar = doc.select(&img_sel).next().and_then(|i| i.value().attr("src")).unwrap_or_default().to_string();

        let mut detail = ActressDetail {
            code: code.to_string(),
            name: String::new(),
            avatar,
            birthday: String::new(), age: String::new(), height: String::new(),
            cup: String::new(), bust: String::new(), waist: String::new(),
            hip: String::new(), hobby: String::new(),
        };

        for p in doc.select(&info_sel) {
            let text = p.text().collect::<String>();
            let text = text.trim();
            if let Some(v) = text.strip_prefix("生日:") { detail.birthday = v.trim().to_string(); }
            else if let Some(v) = text.strip_prefix("年齡:") { detail.age = v.trim().to_string(); }
            else if let Some(v) = text.strip_prefix("身高:") { detail.height = v.trim().to_string(); }
            else if let Some(v) = text.strip_prefix("罩杯:") { detail.cup = v.trim().to_string(); }
            else if let Some(v) = text.strip_prefix("胸圍:") { detail.bust = v.trim().to_string(); }
            else if let Some(v) = text.strip_prefix("腰圍:") { detail.waist = v.trim().to_string(); }
            else if let Some(v) = text.strip_prefix("臀圍:") { detail.hip = v.trim().to_string(); }
            else if let Some(v) = text.strip_prefix("愛好:") { detail.hobby = v.trim().to_string(); }
        }

        // actress name from title
        let title_sel = Selector::parse("title").unwrap();
        if let Some(t) = doc.select(&title_sel).next() {
            let title_text = t.text().collect::<String>();
            detail.name = title_text.split(" - ").next().unwrap_or_default().trim().to_string();
        }

        // parse movies
        let box_sel = Selector::parse(".movie-box").unwrap();
        let date_sel = Selector::parse("date").unwrap();
        let mimg_sel = Selector::parse("img").unwrap();
        let tag_sel = Selector::parse(".item-tag button").unwrap();

        let mut movies = Vec::new();
        for el in doc.select(&box_sel) {
            let link = el.value().attr("href").unwrap_or_default().to_string();
            let dates: Vec<String> = el.select(&date_sel).map(|d| d.text().collect()).collect();
            let number = dates.first().cloned().unwrap_or_default();
            let date = dates.get(1).cloned().unwrap_or_default();
            let title = el.select(&mimg_sel).next().and_then(|i| i.value().attr("title")).unwrap_or_default().to_string();
            let cover = el.select(&mimg_sel).next().and_then(|i| i.value().attr("src")).unwrap_or_default().to_string();
            let tags: Vec<String> = el.select(&tag_sel).map(|t| t.text().collect::<String>().trim().to_string()).filter(|t| !t.is_empty()).collect();
            movies.push(Movie { number, title, link, cover, date, tags });
        }

        let result_info = Self::parse_result_info(&doc);
        let has_next = movies.len() >= 30;
        let page_result = PageResult { movies, page: 1, has_next, result_info };
        Ok((detail, page_result))
    }

    pub async fn fetch_movie_detail(&self, url: &str) -> Result<MovieDetail> {
        let resp = self.client.get(url).send().await?;
        let body = resp.text().await?;
        let doc = Html::parse_document(&body);

        let h3_sel = Selector::parse("h3").unwrap();
        let big_img_sel = Selector::parse(".bigImage img").unwrap();
        let info_sel = Selector::parse(".info p").unwrap();
        let star_sel = Selector::parse(".star-name a").unwrap();
        let genre_sel = Selector::parse(".genre a[href*=\"genre\"]").unwrap();
        let sample_sel = Selector::parse(".sample-box").unwrap();

        let title = doc.select(&h3_sel).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
        let cover = doc.select(&big_img_sel).next().and_then(|e| e.value().attr("src")).unwrap_or_default().to_string();

        let mut number = String::new();
        let mut date = String::new();
        let mut duration = String::new();
        let mut maker = String::new();
        let mut publisher = String::new();

        for p in doc.select(&info_sel) {
            let text = p.text().collect::<String>();
            let text = text.trim().to_string();
            if text.starts_with("識別碼:") { number = text.replace("識別碼:", "").trim().to_string(); }
            else if text.starts_with("發行日期:") { date = text.replace("發行日期:", "").trim().to_string(); }
            else if text.starts_with("長度:") { duration = text.replace("長度:", "").trim().to_string(); }
            else if text.starts_with("製作商:") { maker = text.replace("製作商:", "").trim().to_string(); }
            else if text.starts_with("發行商:") { publisher = text.replace("發行商:", "").trim().to_string(); }
        }

        let genres: Vec<String> = doc.select(&genre_sel).map(|a| a.text().collect::<String>().trim().to_string()).filter(|s| !s.is_empty()).collect();

        let actresses: Vec<Actress> = doc.select(&star_sel).map(|a| {
            let name = a.text().collect::<String>().trim().to_string();
            let link = a.value().attr("href").unwrap_or_default().to_string();
            let code = link.rsplit('/').next().unwrap_or_default().to_string();
            Actress { code, name, avatar: String::new(), link }
        }).collect();

        let sample_images: Vec<String> = doc.select(&sample_sel).filter_map(|a| a.value().attr("href").map(|s| s.to_string())).collect();

        Ok(MovieDetail { number, title, cover, date, duration, maker, publisher, genres, actresses, sample_images })
    }

    pub async fn fetch_star_movies(&self, code: &str, page: usize) -> Result<PageResult> {
        let url = if page > 1 {
            format!("{}/star/{}/{}", self.base_url, code, page)
        } else {
            format!("{}/star/{}", self.base_url, code)
        };
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;

        let doc = Html::parse_document(&body);
        let box_sel = Selector::parse(".movie-box").unwrap();
        let date_sel = Selector::parse("date").unwrap();
        let img_sel = Selector::parse("img").unwrap();
        let tag_sel = Selector::parse(".item-tag button").unwrap();

        let mut movies = Vec::new();
        for el in doc.select(&box_sel) {
            let link = el.value().attr("href").unwrap_or_default().to_string();
            let dates: Vec<String> = el.select(&date_sel).map(|d| d.text().collect()).collect();
            let number = dates.first().cloned().unwrap_or_default();
            let date = dates.get(1).cloned().unwrap_or_default();
            let title = el.select(&img_sel).next().and_then(|i| i.value().attr("title")).unwrap_or_default().to_string();
            let cover = el.select(&img_sel).next().and_then(|i| i.value().attr("src")).unwrap_or_default().to_string();
            let tags: Vec<String> = el.select(&tag_sel).map(|t| t.text().collect::<String>().trim().to_string()).filter(|t| !t.is_empty()).collect();
            movies.push(Movie { number, title, link, cover, date, tags });
        }

        let result_info = Self::parse_result_info(&doc);
        let has_next = movies.len() >= 30;
        Ok(PageResult { movies, page, has_next, result_info })
    }

    fn build_actress_url(&self, keyword: &str, page: usize, uncensored: bool) -> String {
        let prefix = if uncensored { "/uncensored" } else { "" };
        let parent = if uncensored { "uc" } else { "ce" };

        if keyword.is_empty() {
            let mut url = format!("{}{}/actresses", self.base_url, prefix);
            if page > 1 {
                url.push_str(&format!("/{page}"));
            }
            url
        } else {
            format!("{}{}/searchstar/{}&type=&parent={}", self.base_url, prefix, keyword, parent)
        }
    }

    pub async fn fetch_page_ex(&self, keyword: &str, page: usize, uncensored: bool) -> Result<PageResult> {
        let url = self.build_list_url_ex(keyword, page, uncensored);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;

        let doc = Html::parse_document(&body);
        let box_sel = Selector::parse(".movie-box").unwrap();
        let date_sel = Selector::parse("date").unwrap();
        let img_sel = Selector::parse("img").unwrap();
        let tag_sel = Selector::parse(".item-tag button").unwrap();

        let mut movies = Vec::new();
        for el in doc.select(&box_sel) {
            let link = el.value().attr("href").unwrap_or_default().to_string();
            let dates: Vec<String> = el.select(&date_sel).map(|d| d.text().collect()).collect();
            let number = dates.first().cloned().unwrap_or_default();
            let date = dates.get(1).cloned().unwrap_or_default();
            let title = el.select(&img_sel).next().and_then(|i| i.value().attr("title")).unwrap_or_default().to_string();
            let cover = el.select(&img_sel).next().and_then(|i| i.value().attr("src")).unwrap_or_default().to_string();
            let tags: Vec<String> = el.select(&tag_sel).map(|t| t.text().collect::<String>().trim().to_string()).filter(|t| !t.is_empty()).collect();
            movies.push(Movie { number, title, link, cover, date, tags });
        }

        let result_info = Self::parse_result_info(&doc);
        let has_next = movies.len() >= 30;
        Ok(PageResult { movies, page, has_next, result_info })
    }

    fn build_list_url_ex(&self, keyword: &str, page: usize, uncensored: bool) -> String {
        let prefix = if uncensored { "/uncensored" } else { "" };
        let mut url = format!("{}{}", self.base_url, prefix);

        if !keyword.is_empty() {
            if uncensored {
                url.push_str(&format!("/search/{}&type=1", keyword));
            } else {
                url.push_str(&format!("/search/{}", keyword));
            }
        }
        // pagination: /search/keyword/2 or /star/code/2
        if page > 1 {
            url.push_str(&format!("/{page}"));
        }
        url
    }

    fn build_list_url(&self, keyword: &str, page: usize) -> String {
        self.build_list_url_ex(keyword, page, false)
    }

    fn parse_result_info(doc: &Html) -> String {
        let mag_sel = Selector::parse("#resultshowmag").unwrap();
        let all_sel = Selector::parse("#resultshowall").unwrap();
        let mag = doc.select(&mag_sel).next().map(|e| e.text().collect::<String>().trim().to_string());
        let all = doc.select(&all_sel).next().map(|e| e.text().collect::<String>().trim().to_string());
        match (mag, all) {
            (Some(m), Some(a)) => format!("{} / {}", m, a),
            (Some(m), None) => m,
            _ => String::new(),
        }
    }

    fn parse_script_vars(body: &str) -> (String, String, String) {
        let extract = |prefix: &str| -> String {
            body.lines()
                .find(|l| l.contains(prefix))
                .map(|l| {
                    l.replace("var", "")
                        .replace(';', "")
                        .replace(' ', "")
                        .replace('\'', "")
                        .trim()
                        .to_string()
                })
                .unwrap_or_default()
        };

        (extract("var gid"), extract("var uc"), extract("var img"))
    }
}
