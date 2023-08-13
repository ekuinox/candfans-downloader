use anyhow::{anyhow, Result};
use reqwest::{IntoUrl, Method, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize};

pub struct CandfansClient {
    cookie: String,
    x_xsrf_token: String,
}

impl CandfansClient {
    pub fn new(cookie: String, x_xsrf_token: String) -> CandfansClient {
        CandfansClient {
            cookie,
            x_xsrf_token,
        }
    }

    pub fn request(&self, method: Method, url: impl IntoUrl) -> Result<RequestBuilder> {
        let client = reqwest::Client::builder().build()?;
        let request = client
            .request(method, url)
            .header("Cookie", &self.cookie)
            .header("X-Xsrf-Token", &self.x_xsrf_token)
            .header("referer", "https://candfans.jp/");
        Ok(request)
    }

    pub async fn get_user(&self, user_code: &str) -> Result<GetUserSuccessData> {
        let res: CandifansResponse<GetUserSuccessData> = self
            .request(Method::GET, "https://candfans.jp/api/user/get-users")?
            .query(&[("user_code", user_code)])
            .send()
            .await?
            .json()
            .await?;
        res.to_anyhow_result()
    }

    pub async fn get_post(&self, user_id: usize, page_idx: usize) -> Result<Vec<PostData>> {
        let res: CandifansResponse<Vec<PostData>> = self
            .request(Method::GET, "https://candfans.jp/api/contents/get-timeline")?
            .query(&[("user_id", user_id), ("page", page_idx)])
            .send()
            .await?
            .json()
            .await?;
        res.to_anyhow_result()
    }
}

#[derive(Deserialize, Debug)]
#[serde(bound = "T: DeserializeOwned", untagged)]
pub enum CandifansResponse<T: DeserializeOwned> {
    Ok { data: T, status: String },
    Err(ErrorData),
}

impl<T: DeserializeOwned> CandifansResponse<T> {
    pub fn to_anyhow_result(self) -> Result<T> {
        match self {
            Self::Ok { data, .. } => Ok(data),
            Self::Err(err) => Err(err.into()),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ErrorData {
    pub code: String,
    pub errors: serde_json::Value,
    pub message: String,
    pub trace: Vec<String>,
}

impl From<ErrorData> for anyhow::Error {
    fn from(value: ErrorData) -> Self {
        anyhow!("{value:?}")
    }
}

#[derive(Deserialize, Debug)]
pub struct GetUserSuccessData {
    pub plans: Vec<serde_json::Value>,
    pub user: UserData,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    pub id: usize,
    pub movie_cnt: usize,
    pub post_cnt: usize,
    pub username: String,
    pub user_code: String,
}

#[derive(Deserialize, Debug)]
pub struct PlanData {
    pub plan_id: usize,
    pub plan_name: String,
    pub plan_detail: String,
    pub is_joined_plan: bool,
}

#[derive(Deserialize, Debug)]
pub struct PostData {
    pub post_id: usize,
    pub post_type: usize,
    pub user_id: usize,
    pub contents_path1: String,
    pub contents_path2: String,
    pub contents_path3: String,
    pub contents_path4: String,
    pub plans: Vec<PlanData>,
}

impl PostData {
    pub fn paths(&self) -> Vec<&str> {
        [
            self.contents_path1.as_str(),
            self.contents_path2.as_str(),
            self.contents_path3.as_str(),
            self.contents_path4.as_str(),
        ]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect()
    }
}
