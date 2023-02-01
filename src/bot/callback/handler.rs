use teloxide::{requests::Requester, types::CallbackQuery, Bot};

use crate::bot::BotResult;

pub(super) async fn ok(bot: Bot, q: CallbackQuery) -> BotResult {
  bot.send_message(q.from.id, "Хай!").await?;
  bot.answer_callback_query(q.id).await?;
  Ok(())
}

pub(super) async fn delete_message(bot: Bot, q: CallbackQuery) -> BotResult {
  let msg = q.message.unwrap();
  bot.delete_message(msg.chat.id, msg.id).await?;
  Ok(())
}
