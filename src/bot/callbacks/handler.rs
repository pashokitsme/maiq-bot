use teloxide::{payloads::EditMessageTextSetters, requests::Requester, types::CallbackQuery, Bot};

use crate::{
  bot::{notifier::send_to_all, BotResult, DEV_ID},
  db::MongoPool,
};

pub(super) async fn ok(bot: Bot, q: CallbackQuery) -> BotResult {
  bot.answer_callback_query(q.id).await?;
  Ok(())
}

pub(super) async fn delete_message(bot: Bot, q: CallbackQuery) -> BotResult {
  let msg = q.message.unwrap();
  bot.delete_message(msg.chat.id, msg.id).await?;
  Ok(())
}

pub(super) async fn send_broadcast(bot: Bot, q: CallbackQuery, mongo: MongoPool) -> BotResult {
  if q.from.id != *DEV_ID {
    return Ok(());
  }

  let users = mongo.fetch_all_notifiable_ids().await?;
  let msg = q.message.unwrap();
  bot.delete_message(msg.chat.id, msg.id).await?;
  send_to_all(&bot, msg.text().unwrap(), users.as_slice()).await;
  Ok(())
}

pub(super) async fn select_group(bot: Bot, q: CallbackQuery, mongo: MongoPool, group_name: &str) -> BotResult {
  let message = q.message.unwrap();
  let mut user = mongo.get_or_new(message.chat.id).await?;
  user.group = Some(group_name.into());
  user.is_notifications_enabled = true;
  mongo.update(&user).await?;
  bot
    .edit_message_text(message.chat.id, message.id, format!("Теперь твоя группа: <code>{}</code>", user.group.unwrap()))
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;
  Ok(())
}
