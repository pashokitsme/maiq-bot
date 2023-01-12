use std::future::IntoFuture;

use maiq_shared::Snapshot;
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

use super::formatter;

pub async fn try_notify_users(bot: &Bot, mongo: &Mongo, snapshot: &Snapshot) -> Result<(), BotError> {
  let timetables = formatter::separate_to_groups(snapshot);
  let notifiables = db::get_notifiables(&mongo).await?;

  let mut set: JoinSet<Result<teloxide::prelude::Message, RequestError>> = JoinSet::new();

  for noty in notifiables {
    let body = timetables
      .get(&noty.group)
      .map_or(BotError::NoTimetableExpanded { group: noty.group.clone(), snapshot_uid: snapshot.uid.clone() }.to_string(), |x| {
        x.clone()
      });

    for id in noty.user_ids {
      set.spawn(
        bot
          .send_message(ChatId(id), &body)
          .parse_mode(ParseMode::Html)
          .into_future(),
      );
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
