use async_trait::async_trait;
use teloxide::{macros::BotCommands, requests::Requester, types::Message, Bot};

use crate::{
  bot::{BotResult, Dispatch, GlobalState},
  db::MongoPool,
};

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case")]
pub enum DevCommand {
  #[command(description = "")]
  DevNotifiables,
}

#[async_trait]
impl Dispatch for DevCommand {
  type Kind = Message;

  async fn dispatch(self, bot: Bot, kind: Self::Kind, mongo: MongoPool, _state: GlobalState) -> BotResult {
    match self {
      DevCommand::DevNotifiables => bot
        .send_message(kind.from().unwrap().id, format!("{:#?}", mongo.notifiables().await?))
        .await
        .map(|_| ())?,
    }
    Ok(())
  }
}
