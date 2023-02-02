use teloxide::{dispatching::dialogue::InMemStorage, prelude::Dialogue};

pub type GlobalStateStorage = InMemStorage<State>;
pub type GlobalState = Dialogue<State, GlobalStateStorage>;

#[derive(Clone, Default)]
pub enum State {
  #[default]
  None,
}
