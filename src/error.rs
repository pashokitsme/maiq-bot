use maiq_api_wrapper as api;
use teloxide::dispatching::dialogue::InMemStorageError;
use thiserror::Error;

use crate::db::MongoError;

pub trait ReadableError {
  fn readable(&self) -> String;
}

#[derive(Error, Debug)]
pub enum BotError {
  #[error("{command}: invalid command usage")]
  InvalidCommandUsage { command: String, help: String, example: String },

  #[error("api-error: {0}: {1}")]
  ApiError(String, String),

  #[error("mongo-db error: {0}")]
  MongoError(String),

  #[error("teloxide-api error: {0}")]
  TeloxideApiError(teloxide::ApiError),

  #[error("teloxide-request error: {0}")]
  TeloxideRequestError(teloxide::RequestError),

  #[error("storage-error: {0}")]
  TeloxideInMemStorageError(InMemStorageError),
}

impl ReadableError for BotError {
  fn readable(&self) -> String {
    match self {
      BotError::InvalidCommandUsage { command, help, example } => {
        format!(
          "Неправильное использование команды <code>{command}</code>\nИспользование: {help}\nПример: <code>{example}</code>"
        )
      }
      BotError::ApiError(err, desc) => format!("Ошибка API 😓\nПричина: {err}.\nОписание: {desc}"),
      BotError::MongoError(err) => format!("Ошибка MongoDB 😓.\nСообщение: {err}"),
      BotError::TeloxideApiError(err) => format!("Ошибка Teloxide API 😓.\nСообщение: {err}"),
      BotError::TeloxideRequestError(err) => format!("Ошибка Teloxide Request 😓.\nСообщение: {err}"),
      BotError::TeloxideInMemStorageError(err) => format!("Ошибка InMemStorage 😓.\nСообщение: {err}"),
    }
  }
}

impl BotError {
  pub fn invalid_command<T: Into<String>>(command: T, help: T, example: T) -> BotError {
    BotError::InvalidCommandUsage { command: command.into(), help: help.into(), example: example.into() }
  }
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
