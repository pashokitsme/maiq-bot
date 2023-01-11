use chrono::{DateTime, Utc, Weekday};
use maiq_shared::{default::DefaultGroup, Snapshot};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize};

use crate::env;

lazy_static! {
  static ref API_HOST: String = dotenvy::var(env::API_HOST).unwrap();
  static ref TODAY_URL: String = format!("{}/latest/today", *API_HOST);
  static ref NEXT_URL: String = format!("{}/latest/next", *API_HOST);
  static ref POLL_URL: String = format!("{}/poll", *API_HOST);
}

#[derive(Deserialize, Debug)]
pub struct ApiError {
  pub cause: String,
  pub desc: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Poll {
  pub last_update: DateTime<Utc>,
  pub next_update: DateTime<Utc>,
  pub latest_today_uid: Option<String>,
  pub latest_next_uid: Option<String>,
}

impl From<reqwest::Error> for ApiError {
  fn from(e: reqwest::Error) -> Self {
    ApiError { cause: "reqwest".into(), desc: e.to_string() }
  }
}

pub async fn get_latest_today() -> Result<Snapshot, ApiError> {
  get(&*TODAY_URL).await
}

pub async fn get_latest_next() -> Result<Snapshot, ApiError> {
  get(&*NEXT_URL).await
}

pub async fn get_snapshot<T: AsRef<str>>(uid: T) -> Result<Snapshot, ApiError> {
  get(format!("{}/snapshot/{}", *API_HOST, uid.as_ref())).await
}

pub async fn get_default<T: AsRef<str>>(group: T, weekday: Weekday) -> Result<DefaultGroup, ApiError> {
  get(format!("{}/default/{}/{}", *API_HOST, weekday, group.as_ref())).await
}

pub async fn poll() -> Result<Poll, ApiError> {
  get(&*POLL_URL).await
}

async fn get<T: AsRef<str>, O: DeserializeOwned>(url: T) -> Result<O, ApiError> {
  info!("GET {}", url.as_ref());
  let res = reqwest::get(url.as_ref()).await?;
  match res.status() {
    StatusCode::OK => Ok(res.json().await?),
    _ => Err(res.json().await?),
  }
}
