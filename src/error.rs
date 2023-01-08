use teloxide::dispatching::dialogue::InMemStorageError;
use thiserror::Error;

use crate::{api, db::MongoError};

#[derive(Error, Debug)]
pub enum BotError {
  #[error("{0}")]
  InvalidCommandUsage(String),

  #[error("Нет расписания")]
  NoTimetable,

  #[error("Ошибка API:\n<code>{1}</code>")]
  ApiError(String, String),

  #[error("Ошибка MongoDB:\n<code>{0}</code>")]
  MongoError(String),

  #[error("Ошибка TeloxideAPI:\n<code>{0}</code>")]
  TeloxideApiError(teloxide::ApiError),

  #[error("Ошибка TeloxideRequest:\n<code>{0}</code>")]
  TeloxideRequestError(teloxide::RequestError),

  #[error("Ошибка InMemStorage:\n<code>{0}</code>")]
  TeloxideInMemStorageError(InMemStorageError),

  #[error("Ошибка: {0}")]
  Custom(String),
}

impl From<api::ApiError> for BotError {
  fn from(err: api::ApiError) -> Self {
    Self::ApiError(err.cause, err.desc)
  }
}

impl From<teloxide::ApiError> for BotError {
  fn from(err: teloxide::ApiError) -> Self {
    Self::TeloxideApiError(err)
  }
}

impl From<teloxide::RequestError> for BotError {
  fn from(err: teloxide::RequestError) -> Self {
    Self::TeloxideRequestError(err)
  }
}

impl From<InMemStorageError> for BotError {
  fn from(err: InMemStorageError) -> Self {
    Self::TeloxideInMemStorageError(err)
  }
}

impl From<MongoError> for BotError {
  fn from(err: MongoError) -> Self {
    Self::MongoError(err.to_string())
  }
}
