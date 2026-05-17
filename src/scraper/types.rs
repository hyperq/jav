#[derive(Debug, Clone)]
pub struct Movie {
    pub number: String,
    pub title: String,
    pub link: String,
    pub cover: String,
    pub date: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Actress {
    pub code: String,
    pub name: String,
    pub avatar: String,
    pub link: String,
}

#[derive(Debug, Clone)]
pub struct ActressDetail {
    pub code: String,
    pub name: String,
    pub avatar: String,
    pub birthday: String,
    pub age: String,
    pub height: String,
    pub cup: String,
    pub bust: String,
    pub waist: String,
    pub hip: String,
    pub hobby: String,
}

#[derive(Debug, Clone)]
pub struct MovieDetail {
    pub number: String,
    pub title: String,
    pub cover: String,
    pub date: String,
    pub duration: String,
    pub maker: String,
    pub publisher: String,
    pub genres: Vec<String>,
    pub actresses: Vec<Actress>,
    pub sample_images: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Magnet {
    pub link: String,
    pub size: String,
    pub caption: bool,
}

#[derive(Debug)]
pub struct PageResult {
    pub movies: Vec<Movie>,
    pub page: usize,
    pub has_next: bool,
    pub result_info: String,  // "已有磁力 194 / 全部影片 399"
}

#[derive(Debug)]
pub struct ActressPageResult {
    pub actresses: Vec<Actress>,
    pub page: usize,
    pub has_next: bool,
}
