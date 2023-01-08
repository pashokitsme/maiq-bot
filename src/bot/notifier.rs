use std::future::IntoFuture;

use maiq_structs::Snapshot;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, ParseMode},
  Bot, RequestError,
};
use tokio::task::JoinSet;

use crate::{
  db::{self, Mongo},
  error::BotError,
};

use super::timetable;

pub async fn try_notify_users(bot: &Bot, mongo: &Mongo, snapshot: &Snapshot) -> Result<(), BotError> {
  let timetable = timetable::get_formatted_snapshot(snapshot)?;
  let notifiables = db::get_notifiables(&mongo).await?;

  let mut set: JoinSet<Result<teloxide::prelude::Message, RequestError>> = JoinSet::new();

  for noty in notifiables {
    let body = timetable
      .get(&noty.group)
      .map_or(BotError::NoTimetable.to_string(), |x| x.clone());

    for id in noty.user_ids {
      let task = bot.send_message(ChatId(id), &body).parse_mode(ParseMode::Html);
      set.spawn(task.into_future());
    }
  }

  info!("Sending messages to {} users..", set.len());
  while let Some(res) = set.join_next().await {
    let res = res.or_else(|e| Err(BotError::Custom(e.to_string())))?;
    if let Err(err) = res {
      warn!("Error occured while notifying users: {}", err)
    }
  }

  Ok(())
}
