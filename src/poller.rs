use std::time::Duration;

use maiq_shared::utils;
use teloxide::Bot;
use tokio::time::sleep;

use crate::{
  api::{self, Poll},
  bot::notifier,
  db::Mongo,
};

pub struct Poller {
  bot: Bot,
  mongo: Mongo,
  prev: Poll,
}

impl Poller {
  pub async fn new(bot: Bot, mongo: Mongo) -> Self {
    let first_poll = api::poll().await.expect("Couldn't make a first poll");
    Self { bot, mongo, prev: first_poll }
  }

  pub async fn run(&mut self) {
    loop {
      self.wait().await;
      let poll = match api::poll().await {
        Ok(p) => p,
        Err(err) => {
          error!("Couldn't make a poll request: {}: {}", err.cause, err.desc);
          sleep(Duration::from_secs(10)).await;
          continue;
        }
      };

      if self.prev.latest_today_uid != poll.latest_today_uid && poll.latest_today_uid.is_some() {
        self.notify(poll.latest_today_uid.as_ref().unwrap().as_str()).await;
      }

      if self.prev.latest_next_uid != poll.latest_next_uid && poll.latest_next_uid.is_some() {
        self.notify(poll.latest_next_uid.as_ref().unwrap().as_str()).await;
      }

      self.prev = poll;
    }
  }

  async fn notify<'a>(&self, uid: &'a str) {
    info!("Trying to send snapshot {} to users", uid);
    let snapshot = match api::snapshot(uid).await {
      Ok(s) => s,
      Err(err) => {
        error!("Snapshot {} returned with error: {}: {}", uid, err.cause, err.desc);
        return;
      }
    };

    if let Err(err) = notifier::try_notify_users(&self.bot, &self.mongo, &snapshot).await {
      error!("An error occured while notifying users: {}", err);
    }
  }

  async fn wait(&self) {
    let wait_for = self
      .prev
      .next_update
      .signed_duration_since(utils::now(0))
      .num_milliseconds() as u64;
    info!("Waiting for {}s for next update", wait_for as f32 / 1000f32);
    sleep(Duration::from_millis(wait_for)).await;
  }
}
