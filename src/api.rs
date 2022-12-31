#![allow(dead_code)]

use chrono::{DateTime, Utc};
use maiq_structs::Snapshot;
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
  pub last_updated: Option<DateTime<Utc>>,
  pub next_update: Option<DateTime<Utc>>,
  pub latest_today_uid: Option<String>,
  pub latest_next_uid: Option<String>,
}

impl From<reqwest::Error> for ApiError {
  fn from(e: reqwest::Error) -> Self {
    ApiError { cause: "reqwest".into(), desc: e.to_string() }
  }
}

pub async fn get_latest_today() -> Result<Snapshot, ApiError> {
  info!("Get today timetable");
  get(&*TODAY_URL).await
}

pub async fn get_latest_next() -> Result<Snapshot, ApiError> {
  info!("Get next timetable");
  get(&*NEXT_URL).await
}

pub async fn get_snapshot<T: AsRef<str>>(uid: T) -> Result<Snapshot, ApiError> {
  info!("Get snapshot {}", uid.as_ref());
  get(format!("{}/snapshot/{}", *API_HOST, uid.as_ref())).await
}

pub async fn poll() -> Result<Poll, ApiError> {
  info!("Polling..");
  get(&*POLL_URL).await
}

async fn get<T: AsRef<str>, O: DeserializeOwned>(url: T) -> Result<O, ApiError> {
  let res = reqwest::get(url.as_ref()).await?;
  match res.status() {
    StatusCode::OK => Ok(res.json().await?),
    _ => Err(res.json().await?),
  }
}
