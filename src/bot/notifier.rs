use std::future::IntoFuture;

use chrono::Datelike;
use maiq_shared::Snapshot;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, ParseMode},
  Bot, RequestError,
};
use tokio::task::JoinSet;

use crate::{
  api::{self, InnerPoll},
  db::{self, Mongo, Notifiable},
  error::BotError,
};

use super::formatter;

pub async fn try_notify_users(bot: &Bot, mongo: &Mongo, prev: &Option<&InnerPoll>, snapshot: &Snapshot) -> Result<(), BotError> {
  let timetables = formatter::separate_to_groups(snapshot, prev);
  let notifiables = db::get_notifiables(&mongo).await?;

  let mut handles: JoinSet<Result<teloxide::prelude::Message, RequestError>> = JoinSet::new();

  for noty in notifiables {
    let body = get_body(&noty).await;

    for id in noty.user_ids {
      handles.spawn(
        bot
          .send_message(ChatId(id), &body)
          .parse_mode(ParseMode::Html)
          .into_future(),
      );
    }
  }

  info!("Sending messages to {} users..", handles.len());
  while let Some(res) = handles.join_next().await {
    let res = res.or_else(|e| Err(BotError::Custom(e.to_string())))?;
    if let Err(err) = res {
      warn!("Error occured while notifying users: {}", err)
    }
  }

  async fn get_body(noty: &Notifiable) -> String {
    match timetables.get(&noty.group) {
      Some(body) => body,
      None => {
        api::get_default(&noty.group, snapshot.date.weekday()).await;
      }
    }
  }

  Ok(())
}
