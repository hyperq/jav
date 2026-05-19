use anyhow::{bail, Result};
use reqwest::header::{COOKIE, REFERER, USER_AGENT};

use super::types::*;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";

pub struct Client115 {
    client: reqwest::Client,
    cookie: SavedCookie,
}

impl Client115 {
    pub fn new(cookie: SavedCookie) -> Result<Self> {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(USER_AGENT, UA.parse()?);
                h.insert(REFERER, "https://115.com/".parse()?);
                h
            })
            .build()?;
        Ok(Self { client, cookie })
    }

    pub fn cookie(&self) -> &SavedCookie {
        &self.cookie
    }

    async fn get_offline_sign(&self) -> Result<(String, i64)> {
        let resp: OfflineSpaceResp = self.client
            .get("https://115.com/?ct=offline&ac=space")
            .header(COOKIE, self.cookie.to_header())
            .send()
            .await?
            .json()
            .await?;

        let sign = resp.sign.ok_or_else(|| anyhow::anyhow!("获取 offline sign 失败"))?;
        let time = resp.time.ok_or_else(|| anyhow::anyhow!("获取 offline time 失败"))?;
        Ok((sign, time))
    }

    async fn get_uid(&self) -> Result<i64> {
        let resp: UserInfoResp = self.client
            .get("https://my.115.com/?ct=ajax&ac=get_user_aq")
            .header(COOKIE, self.cookie.to_header())
            .send()
            .await?
            .json()
            .await?;

        resp.data
            .and_then(|d| d.uid)
            .ok_or_else(|| anyhow::anyhow!("获取用户 UID 失败"))
    }

    pub async fn get_quota(&self) -> Result<CloudQuota> {
        let (sign, time) = self.get_offline_sign().await?;
        let uid = self.get_uid().await?;
        let resp: QuotaResp = self.client
            .post("https://115.com/web/lixian/?ct=lixian&ac=task_lists")
            .header(COOKIE, self.cookie.to_header())
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&[
                ("uid", uid.to_string()),
                ("sign", sign),
                ("time", time.to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;
        Ok(CloudQuota {
            quota: resp.quota.unwrap_or(0),
            total: resp.total.unwrap_or(0),
        })
    }

    pub async fn add_task(&self, magnet_url: &str, wp_path_id: &str) -> Result<String> {
        let (sign, time) = self.get_offline_sign().await?;
        let uid = self.get_uid().await?;

        let mut params = vec![
            ("url", magnet_url.to_string()),
            ("uid", uid.to_string()),
            ("sign", sign),
            ("time", time.to_string()),
        ];

        if !wp_path_id.is_empty() {
            params.push(("wp_path_id", wp_path_id.to_string()));
        }

        let resp: AddTaskResp = self.client
            .post("https://115.com/web/lixian/?ct=lixian&ac=add_task_url")
            .header(COOKIE, self.cookie.to_header())
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&params)
            .send()
            .await?
            .json()
            .await?;

        if resp.state {
            Ok(resp.name.unwrap_or_else(|| "未知任务".into()))
        } else {
            let msg = resp.error_msg.unwrap_or_else(|| "未知错误".into());
            bail!("{msg}")
        }
    }

    pub async fn list_tasks(&self, page: usize) -> Result<Vec<OfflineTask>> {
        let (sign, time) = self.get_offline_sign().await?;
        let uid = self.get_uid().await?;

        let resp: TaskListResp = self.client
            .post("https://115.com/web/lixian/?ct=lixian&ac=task_lists")
            .header(COOKIE, self.cookie.to_header())
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&[
                ("uid", uid.to_string()),
                ("sign", sign),
                ("time", time.to_string()),
                ("page", page.to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.tasks.unwrap_or_default())
    }

    pub async fn list_files(&self, cid: &str) -> Result<Vec<FileInfo>> {
        let resp: FilesResp = self.client
            .get("https://webapi.115.com/files")
            .header(COOKIE, self.cookie.to_header())
            .query(&[("cid", cid), ("show_dir", "0"), ("nf", "1")])
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.data.unwrap_or_default())
    }

    pub async fn get_video_play_url(&self, pick_code: &str) -> Result<(String, String)> {
        let key = super::m115::generate_key();
        let params = serde_json::json!({"pickcode": pick_code}).to_string();
        let data = super::m115::encode(params.as_bytes(), &key);

        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let resp: serde_json::Value = self.client
            .post(format!("https://proapi.115.com/app/chrome/downurl?t={t}"))
            .header(COOKIE, self.cookie.to_header())
            .form(&[("data", &data)])
            .send()
            .await?
            .json()
            .await?;

        let state = resp.get("state").and_then(|s| s.as_bool()).unwrap_or(false);
        if !state {
            let msg = resp.get("msg").and_then(|m| m.as_str()).unwrap_or("未知错误");
            bail!("115 下载接口: {msg}");
        }

        let encoded_data = resp.get("data").and_then(|d| d.as_str())
            .ok_or_else(|| anyhow::anyhow!("响应缺少 data 字段"))?;

        let decrypted = super::m115::decode(encoded_data, &key)?;
        let info: serde_json::Value = serde_json::from_slice(&decrypted)?;

        if let Some(obj) = info.as_object() {
            for (_k, v) in obj {
                if let Some(url) = v.pointer("/url/url").and_then(|u| u.as_str()) {
                    if !url.is_empty() {
                        return Ok((url.to_string(), UA.to_string()));
                    }
                }
            }
        }
        bail!("115 未返回可用的下载链接, decoded: {}", String::from_utf8_lossy(&decrypted[..decrypted.len().min(300)]))
    }

    pub async fn search_files(&self, keyword: &str) -> Result<Vec<FileInfo>> {
        let resp: SearchFilesResp = self.client
            .get("https://webapi.115.com/files/search")
            .header(COOKIE, self.cookie.to_header())
            .query(&[
                ("search_value", keyword),
                ("offset", "0"),
                ("limit", "50"),
                ("aid", "1"),
                ("cid", "0"),
                ("show_dir", "1"),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.data.unwrap_or_default())
    }

    fn find_task<'a>(tasks: &'a [OfflineTask], info_hash: &str) -> Option<&'a OfflineTask> {
        tasks.iter().find(|t| {
            t.info_hash.as_deref()
                .map(|h| h.eq_ignore_ascii_case(info_hash))
                .unwrap_or(false)
        })
    }

    fn find_video_pick_code(files: &[FileInfo]) -> Option<&str> {
        let video_exts = ["mp4", "mkv", "avi", "wmv", "flv", "mov", "ts", "rmvb"];
        files.iter()
            .filter(|f| {
                f.file_name.as_deref()
                    .map(|n| video_exts.iter().any(|ext| n.to_lowercase().ends_with(ext)))
                    .unwrap_or(false)
            })
            .max_by_key(|f| f.size.unwrap_or(0))
            .or_else(|| files.iter().max_by_key(|f| f.size.unwrap_or(0)))
            .and_then(|f| f.pick_code.as_deref())
    }

    /// Full play flow: submit → poll → get URL.
    /// Falls back to file search if task already exists or not in recent list.
    pub async fn play_flow(
        &self,
        magnet_link: &str,
        info_hash: &str,
        search_keyword: &str,
        on_progress: impl Fn(String),
    ) -> Result<(String, String)> {
        on_progress("🔍 检查 115 离线任务...".into());
        let tasks = self.list_tasks(1).await?;

        let task = Self::find_task(&tasks, info_hash);
        match task.and_then(|t| t.status) {
            Some(2) => {
                on_progress("✅ 任务已完成，获取播放链接...".into());
                return self.get_url_from_task(info_hash, &on_progress).await;
            }
            Some(_) => {
                on_progress("⏳ 任务下载中，等待完成...".into());
                self.poll_task_completion(info_hash, &on_progress).await?;
                return self.get_url_from_task(info_hash, &on_progress).await;
            }
            None => {
                // not in task list — try submit, may fail with "already exists"
                on_progress("📤 提交磁链到 115 离线下载...".into());
                match self.add_task(magnet_link, "").await {
                    Ok(_) => {
                        on_progress("⏳ 已提交，等待下载完成...".into());
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        self.poll_task_completion(info_hash, &on_progress).await?;
                        return self.get_url_from_task(info_hash, &on_progress).await;
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("已存在") || msg.contains("exist") {
                            on_progress("📁 任务已存在，搜索云盘文件...".into());
                            // fall through to search
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // fallback: search 115 cloud by keyword
        self.get_url_by_search(search_keyword, &on_progress).await
    }

    async fn get_url_from_task(
        &self,
        info_hash: &str,
        on_progress: &impl Fn(String),
    ) -> Result<(String, String)> {
        on_progress("📂 获取文件信息...".into());
        let tasks = self.list_tasks(1).await?;
        let task = Self::find_task(&tasks, info_hash)
            .ok_or_else(|| anyhow::anyhow!("任务丢失"))?;
        let file_id = task.file_id.as_deref()
            .ok_or_else(|| anyhow::anyhow!("任务无文件ID"))?;

        let files = self.list_files(file_id).await?;
        let pick_code = Self::find_video_pick_code(&files)
            .ok_or_else(|| anyhow::anyhow!("未找到视频文件"))?;

        on_progress("🔗 获取播放链接...".into());
        self.get_video_play_url(pick_code).await
    }

    async fn get_url_by_search(
        &self,
        keyword: &str,
        on_progress: &impl Fn(String),
    ) -> Result<(String, String)> {
        on_progress(format!("🔍 搜索 115 云盘: {keyword}"));
        let files = self.search_files(keyword).await?;

        let pick_code = Self::find_video_pick_code(&files)
            .ok_or_else(|| anyhow::anyhow!("115 云盘未找到匹配的视频文件"))?;

        on_progress("🔗 获取播放链接...".into());
        self.get_video_play_url(pick_code).await
    }

    async fn poll_task_completion(
        &self,
        info_hash: &str,
        on_progress: &impl Fn(String),
    ) -> Result<()> {
        let max_polls = 120; // 120 * 3s = 6min max wait
        for _ in 0..max_polls {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            let tasks = self.list_tasks(1).await?;
            if let Some(task) = Self::find_task(&tasks, info_hash) {
                let pct = task.percent.unwrap_or(0.0);
                match task.status {
                    Some(2) => return Ok(()),
                    Some(-1) => bail!("115 离线下载失败"),
                    _ => {
                        on_progress(format!("⏳ 115 下载中: {:.0}%", pct));
                    }
                }
            } else {
                // task may appear with delay after submission
                on_progress("⏳ 等待任务出现...".into());
            }
        }
        bail!("等待超时，请稍后用 P 键重试")
    }

    /// Batch submit up to 15 URLs per call via add_task_urls
    pub async fn add_tasks(&self, urls: &[String], wp_path_id: &str) -> Result<BatchAddResult> {
        let (sign, time) = self.get_offline_sign().await?;
        let uid = self.get_uid().await?;

        let mut result = BatchAddResult { ok: 0, fail: 0, errors: vec![] };

        for chunk in urls.chunks(15) {
            let mut params: Vec<(String, String)> = vec![
                ("uid".into(), uid.to_string()),
                ("sign".into(), sign.clone()),
                ("time".into(), time.to_string()),
            ];
            if !wp_path_id.is_empty() {
                params.push(("wp_path_id".into(), wp_path_id.to_string()));
            }
            for (i, url) in chunk.iter().enumerate() {
                params.push((format!("url[{i}]"), url.clone()));
            }

            let resp: AddTasksResp = self.client
                .post("https://115.com/web/lixian/?ct=lixian&ac=add_task_urls")
                .header(COOKIE, self.cookie.to_header())
                .header("X-Requested-With", "XMLHttpRequest")
                .form(&params)
                .send()
                .await?
                .json()
                .await?;

            if resp.state {
                for item in resp.result.unwrap_or_default() {
                    if item.state {
                        result.ok += 1;
                    } else {
                        result.fail += 1;
                        if let Some(msg) = item.error_msg {
                            result.errors.push(msg);
                        }
                    }
                }
            } else {
                result.fail += chunk.len();
                if let Some(msg) = resp.error_msg {
                    result.errors.push(msg);
                }
            }
        }
        Ok(result)
    }
}

pub struct BatchAddResult {
    pub ok: usize,
    pub fail: usize,
    pub errors: Vec<String>,
}
