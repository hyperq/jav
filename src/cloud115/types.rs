#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QrTokenResp {
    #[serde(default)]
    pub state: i32,
    pub data: Option<QrTokenData>,
}

#[derive(Debug, Deserialize)]
pub struct QrTokenData {
    pub uid: String,
    pub time: i64,
    pub sign: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QrStatus {
    Waiting,
    Scanned,
    Confirmed,
    Expired,
    Canceled,
    Unknown(i32),
}

impl QrStatus {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Waiting,
            1 => Self::Scanned,
            2 => Self::Confirmed,
            -1 => Self::Expired,
            -2 => Self::Canceled,
            c => Self::Unknown(c),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Waiting => "等待扫码...",
            Self::Scanned => "已扫码，请在手机上确认",
            Self::Confirmed => "登录成功！",
            Self::Expired => "二维码已过期，请刷新",
            Self::Canceled => "已取消",
            Self::Unknown(_) => "未知状态",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct QrStatusResp {
    #[serde(default)]
    pub state: i32,
    pub data: Option<QrStatusData>,
}

#[derive(Debug, Deserialize)]
pub struct QrStatusData {
    pub status: i32,
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginResp {
    #[serde(default)]
    pub state: i32,
    pub data: Option<LoginData>,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub cookie: Option<LoginCookie>,
    pub user_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LoginCookie {
    #[serde(rename = "UID")]
    pub uid: Option<String>,
    #[serde(rename = "CID")]
    pub cid: Option<String>,
    #[serde(rename = "SEID")]
    pub seid: Option<String>,
    #[serde(rename = "KID")]
    pub kid: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct SavedCookie {
    pub uid: String,
    pub cid: String,
    pub seid: String,
    pub kid: String,
}

impl SavedCookie {
    pub fn to_header(&self) -> String {
        format!("UID={}; CID={}; SEID={}; KID={}", self.uid, self.cid, self.seid, self.kid)
    }
}

#[derive(Debug, Deserialize)]
pub struct OfflineSpaceResp {
    #[serde(default)]
    pub state: bool,
    pub sign: Option<String>,
    pub time: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfoResp {
    #[serde(default)]
    pub state: bool,
    pub data: Option<UserInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfoData {
    pub uid: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AddTaskResp {
    #[serde(default)]
    pub state: bool,
    pub name: Option<String>,
    pub error_msg: Option<String>,
    pub errcode: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct QuotaResp {
    pub quota: Option<i64>,
    pub total: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct CloudQuota {
    pub quota: i64, // remaining
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddTasksResp {
    #[serde(default)]
    pub state: bool,
    pub error_msg: Option<String>,
    pub result: Option<Vec<AddTaskItem>>,
}

#[derive(Debug, Deserialize)]
pub struct AddTaskItem {
    #[serde(default)]
    pub state: bool,
    pub error_msg: Option<String>,
}

// --- 离线任务列表 ---

#[derive(Debug, Deserialize)]
pub struct TaskListResp {
    #[serde(default)]
    pub state: bool,
    pub tasks: Option<Vec<OfflineTask>>,
    pub count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OfflineTask {
    pub info_hash: Option<String>,
    pub name: Option<String>,
    pub status: Option<i32>,
    pub file_id: Option<String>,
    pub percent: Option<f64>,
    pub size: Option<i64>,
    pub url: Option<String>,
}

// --- 文件列表 ---

#[derive(Debug, Deserialize)]
pub struct FilesResp {
    #[serde(default)]
    pub state: bool,
    pub data: Option<Vec<FileInfo>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub fid: Option<String>,
    #[serde(rename = "pc")]
    pub pick_code: Option<String>,
    #[serde(rename = "n")]
    pub file_name: Option<String>,
    #[serde(rename = "s", default, deserialize_with = "deserialize_size")]
    pub size: Option<i64>,
    pub ico: Option<String>,
}

fn deserialize_size<'de, D>(de: D) -> Result<Option<i64>, D::Error>
where D: serde::Deserializer<'de> {
    let v: Option<serde_json::Value> = Option::deserialize(de)?;
    Ok(v.and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }))
}

// --- 文件搜索 ---

#[derive(Debug, Deserialize)]
pub struct SearchFilesResp {
    #[serde(default)]
    pub state: bool,
    pub count: Option<i32>,
    pub data: Option<Vec<FileInfo>>,
}

// --- 下载 URL ---

#[derive(Debug, Deserialize)]
pub struct DownloadUrlResp {
    #[serde(default)]
    pub state: bool,
    pub data: Option<serde_json::Value>,
}
