use teloxide::{
  dispatching::{dialogue::InMemStorage, DpHandlerDescription},
  prelude::*,
};

use crate::bot::handler_context::Ok;

type SetupDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(Clone, Default)]
pub enum State {
  #[default]
  Start,
  GroupName {
    group: String,
  },
}

pub fn build() -> Handler<'static, DependencyMap, Ok, DpHandlerDescription> {
  Update::filter_message()
    .enter_dialogue::<Message, InMemStorage<State>, State>()
    .branch(dptree::case![State::Start].endpoint(start))
}

async fn start(bot: Bot, dlg: SetupDialogue, msg: Message) -> Ok {
  bot.send_message(msg.chat.id, "Привет. Введи свою группу").await?;
  Ok(())
}
