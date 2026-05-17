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
