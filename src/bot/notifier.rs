use std::{future::IntoFuture, time::Duration};

use maiq_shared::Snapshot;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, ParseMode},
  Bot,
};
use tokio::task::JoinSet;

use crate::{
  api::InnerPoll,
  bot::format::{Change, SnapshotFormatter, SnapshotFormatterExt},
  db::MongoPool,
  error::BotError,
};

pub async fn notify_users(bot: &Bot, mongo: &MongoPool, prev: &Option<&InnerPoll>, snapshot: &Snapshot) -> Result<(), BotError> {
  let changes = snapshot.lookup_changes(prev);
  info!("Changes: {:?}", changes);
  let notifiables = mongo.notifiables().await?;

  for noty in notifiables {
    match changes.get(&*noty.group) {
      Some(kind) if *kind == Change::Nothing => continue,
      None => continue,
      Some(_) => (),
    }

    let body = snapshot
      .format_or_default(&*noty.group, snapshot.date.date_naive())
      .await;

    send_to_all(&bot, &*body, &noty.user_ids.as_slice()).await?;
  }
  Ok(())
}

pub async fn send_to_all(bot: &Bot, msg: &str, ids: &[i64]) -> Result<(), BotError> {
  let mut handles = JoinSet::new();

  for &id in ids {
    handles.spawn(
      bot
        .send_message(ChatId(id), msg)
        .parse_mode(ParseMode::Html)
        .into_future(),
    );
  }

  info!("Sending message to users {:?} ({})..", ids, ids.len());
  for (idx, handle) in handles.join_next().await.iter().enumerate() {
    if let Err(err) = handle {
      warn!("Error occured while notifying users at [{}]: {}", idx, err)
    }

    if let Err(err) = handle.as_ref().unwrap() {
      warn!("Error occured while notifying users at [{}]: {}", idx, err)
    }

    if idx % 25 == 0 {
      tokio::time::sleep(Duration::from_secs(1)).await
    }
  }
  Ok(())
}
