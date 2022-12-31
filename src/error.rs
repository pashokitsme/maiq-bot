use teloxide::dispatching::dialogue::InMemStorageError;
use thiserror::Error;

use crate::api;

#[derive(Error, Debug)]
pub enum TeloxideError {
  #[error("Ошибка: API: {0}. {1}.")]
  ApiError(String, String),

  #[error("Ошибка: TeloxideAPI: {0}")]
  TeloxideApiError(teloxide::ApiError),

  #[error("Ошибка: TeloxideRequest: {0}")]
  TeloxideRequestError(teloxide::RequestError),

  #[error("Ошибка: InMemStorageError: {0}")]
  TeloxideInMemStorageError(InMemStorageError),
}

impl From<api::ApiError> for TeloxideError {
  fn from(err: api::ApiError) -> Self {
    Self::ApiError(err.cause, err.desc)
  }
}

impl From<teloxide::ApiError> for TeloxideError {
  fn from(err: teloxide::ApiError) -> Self {
    Self::TeloxideApiError(err)
  }
}

impl From<teloxide::RequestError> for TeloxideError {
  fn from(err: teloxide::RequestError) -> Self {
    Self::TeloxideRequestError(err)
  }
}

impl From<InMemStorageError> for TeloxideError {
  fn from(err: InMemStorageError) -> Self {
    Self::TeloxideInMemStorageError(err)
  }
}
