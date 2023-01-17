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
  api::InnerPoll,
  bot::format::{Change, SnapshotFormatter, SnapshotFormatterExt},
  db::{self, Mongo},
  error::BotError,
};

pub async fn try_notify_users(bot: &Bot, mongo: &Mongo, prev: &Option<&InnerPoll>, snapshot: &Snapshot) -> Result<(), BotError> {
  let changes = snapshot.lookup_changes(prev);
  info!("Changes: {:?}", changes);
  let notifiables = db::get_notifiables(&mongo).await?;

  let mut handles: JoinSet<Result<teloxide::prelude::Message, RequestError>> = JoinSet::new();

  for noty in notifiables {
    match changes.get(&*noty.group) {
      Some(kind) if *kind == Change::Nothing => continue,
      None => continue,
      Some(_) => (),
    }

    let body = snapshot
      .format_or_default(&*noty.group, snapshot.date.date_naive())
      .await;

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
  
  Ok(())
}
