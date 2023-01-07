use maiq_structs::Snapshot;
use teloxide::{
  payloads::SendMessageSetters,
  requests::Requester,
  types::{ChatId, ParseMode},
  Bot,
};

use crate::{
  db::{self, Mongo},
  error::BotError,
};

use super::timetable;

pub async fn process_notify_users(bot: &Bot, mongo: &Mongo, snapshot: &Snapshot) -> Result<(), BotError> {
  let timetable = timetable::get_formatted_snapshot(snapshot)?;
  let notifiables = db::get_notifiables(&mongo).await?;

  for noty in notifiables {
    let body = timetable
      .get(&noty.group)
      .map_or(BotError::NoTimetable.to_string(), |x| x.clone());

    for id in noty.user_ids {
      bot
        .send_message(ChatId(id), &body)
        .parse_mode(ParseMode::Html)
        .await?;
    }
  }

  Ok(())
}
