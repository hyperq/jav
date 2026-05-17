use anyhow::{bail, Result};
use reqwest::header::{COOKIE, USER_AGENT};

use super::types::*;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";

pub struct QrLogin {
    client: reqwest::Client,
    token: Option<QrTokenData>,
}

impl QrLogin {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(USER_AGENT, UA.parse()?);
                h
            })
            .build()?;
        Ok(Self { client, token: None })
    }

    pub async fn request_token(&mut self) -> Result<&QrTokenData> {
        let resp: QrTokenResp = self.client
            .get("https://qrcodeapi.115.com/api/1.0/web/1.0/token/")
            .send()
            .await?
            .json()
            .await?;

        match resp.data {
            Some(data) => {
                self.token = Some(data);
                Ok(self.token.as_ref().unwrap())
            }
            None => bail!("获取二维码 token 失败"),
        }
    }

    pub fn qr_image_url(&self) -> Option<String> {
        self.token.as_ref().map(|t| {
            format!("https://qrcodeapi.115.com/api/1.0/web/1.0/qrcode?uid={}", t.uid)
        })
    }

    pub async fn fetch_qr_image(&self) -> Result<Vec<u8>> {
        let url = self.qr_image_url().ok_or_else(|| anyhow::anyhow!("未获取 token"))?;
        let bytes = self.client.get(&url).send().await?.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn poll_status(&self) -> Result<QrStatus> {
        let token = self.token.as_ref().ok_or_else(|| anyhow::anyhow!("未获取 token"))?;
        let url = format!(
            "https://qrcodeapi.115.com/get/status/?uid={}&time={}&sign={}",
            token.uid, token.time, token.sign
        );
        let resp: QrStatusResp = self.client.get(&url).send().await?.json().await?;
        let code = resp.data.map(|d| d.status).unwrap_or(-99);
        Ok(QrStatus::from_code(code))
    }

    pub async fn finish_login(&self) -> Result<SavedCookie> {
        let token = self.token.as_ref().ok_or_else(|| anyhow::anyhow!("未获取 token"))?;
        let resp: LoginResp = self.client
            .post("https://passportapi.115.com/app/1.0/web/1.0/login/qrcode/")
            .form(&[("account", token.uid.as_str()), ("app", "web")])
            .send()
            .await?
            .json()
            .await?;

        let data = resp.data.ok_or_else(|| anyhow::anyhow!("登录响应无 data"))?;
        let cookie = data.cookie.ok_or_else(|| anyhow::anyhow!("登录响应无 cookie"))?;

        Ok(SavedCookie {
            uid: cookie.uid.unwrap_or_default(),
            cid: cookie.cid.unwrap_or_default(),
            seid: cookie.seid.unwrap_or_default(),
            kid: cookie.kid.unwrap_or_default(),
        })
    }
}

pub fn cookie_path() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| ".".into());
    home.join(".jav").join("115_cookie.json")
}

pub fn save_cookie(cookie: &SavedCookie) -> Result<()> {
    let path = cookie_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(cookie)?;
    std::fs::write(&path, json)?;
    Ok(())
}

pub fn load_cookie() -> Option<SavedCookie> {
    let path = cookie_path();
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub async fn check_login(cookie: &SavedCookie) -> bool {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://my.115.com/?ct=guide&ac=status")
        .header(USER_AGENT, UA)
        .header(COOKIE, cookie.to_header())
        .send()
        .await;

    match resp {
        Ok(r) => {
            let body: serde_json::Value = r.json().await.unwrap_or_default();
            body.get("state").and_then(|s| s.as_bool()).unwrap_or(false)
        }
        Err(_) => false,
    }
}
