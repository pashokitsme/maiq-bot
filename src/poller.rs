use std::time::Duration;

use chrono::NaiveTime;
use maiq_shared::utils;
use teloxide::Bot;
use tokio::time::sleep;

use crate::{
  api::{self, InnerPoll, Poll},
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

      if utils::now(0).time() < NaiveTime::from_hms_opt(6, 0, 0).unwrap() {
        info!("Skipping due to the night");
        sleep(Duration::from_secs(6 * 60 * 60)).await;
        continue;
      }

      let poll = match api::poll().await {
        Ok(p) => p,
        Err(err) => {
          error!("Couldn't make a poll request: {}: {}", err.cause, err.desc);
          sleep(Duration::from_secs(10)).await;
          continue;
        }
      };

      if self.is_notify_needed(self.prev.today.as_ref(), poll.today.as_ref()) {
        self
          .notify(poll.today.as_ref().unwrap().uid.as_str(), &self.prev.today.as_ref())
          .await;
      }

      if self.is_notify_needed(self.prev.next.as_ref(), poll.next.as_ref()) {
        self
          .notify(poll.next.as_ref().unwrap().uid.as_str(), &self.prev.next.as_ref())
          .await;
      }

      self.prev = poll;
    }
  }

  fn is_notify_needed(&self, prev: Option<&InnerPoll>, poll: Option<&InnerPoll>) -> bool {
    match (prev, poll) {
      (None, None) => false,
      (None, Some(_)) => true,
      (Some(_), None) => false,
      (Some(a), Some(b)) => a.uid != b.uid,
    }
  }

  async fn notify<'a>(&self, uid: &'a str, prev: &Option<&InnerPoll>) {
    info!("Trying to send snapshot {} to users", uid);
    let snapshot = match api::snapshot(uid).await {
      Ok(s) => s,
      Err(err) => {
        error!("Snapshot {} returned with error: {}: {}", uid, err.cause, err.desc);
        return;
      }
    };

    if let Err(err) = notifier::try_notify_users(&self.bot, &self.mongo, prev, &snapshot).await {
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
