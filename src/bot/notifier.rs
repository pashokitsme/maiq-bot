use std::{future::IntoFuture, time::Duration};

use maiq_api_wrapper::polling::SnapshotChanges;
use maiq_shared::Snapshot;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, ParseMode},
  Bot,
};
use tokio::task::JoinSet;

use crate::{bot::format::SnapshotFormatterExt, db::MongoPool, error::BotError};

pub async fn notify_update(bot: &Bot, mongo: &MongoPool, changes: &SnapshotChanges, snapshot: Snapshot) -> Result<(), BotError> {
  info!("Changes: {:?}", changes);
  let notifiables = mongo.notifiables().await?;

  for notifiable in notifiables {
    match changes.groups.get(&*notifiable.group) {
      Some(kind) if kind.is_same() => continue,
      None => continue,
      _ => (),
    }

    let body = snapshot
      .format_or_default(&*notifiable.group, snapshot.date.date_naive())
      .await;

    send_to_all(&bot, &*body, &notifiable.user_ids.as_slice()).await;
  }
  Ok(())
}

pub async fn send_to_all(bot: &Bot, msg: &str, ids: &[i64]) {
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
      warn!("Error occured while notifying users at [{}]: {}", idx, err);
    }

    if let Err(req_err) = handle.as_ref().unwrap() {
      warn!("Request error occured while notifying users at [{}]: {}", idx, req_err);
    }

    if idx % 25 == 0 {
      tokio::time::sleep(Duration::from_secs(1)).await
    }
  }

  info!("Sending done");
}
