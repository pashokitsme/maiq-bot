use std::time::Duration;

use chrono::NaiveTime;
use maiq_shared::utils;
use teloxide::Bot;
use tokio::time::sleep;

use crate::{
  api::{self, InnerPoll, Poll},
  bot::notifier,
  db::MongoPool,
};

pub struct Poller {
  bot: Bot,
  mongo: MongoPool,
  prev: Poll,
}

impl Poller {
  pub async fn new(bot: Bot, mongo: MongoPool) -> Self {
    let first_poll = api::poll().await.expect("Couldn't make a first poll");
    info!("First poll: {:?}", first_poll);
    Self { bot, mongo, prev: first_poll }
  }

  pub async fn run(&mut self) {
    loop {
      let is_should_be_silent = self.wait().await;

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
          .notify(poll.today.as_ref().unwrap().uid.as_str(), &self.prev.today.as_ref(), is_should_be_silent)
          .await;
      }

      if self.is_notify_needed(self.prev.next.as_ref(), poll.next.as_ref()) {
        self
          .notify(poll.next.as_ref().unwrap().uid.as_str(), &self.prev.next.as_ref(), is_should_be_silent)
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

  async fn notify(&self, uid: &str, prev: &Option<&InnerPoll>, silent: bool) {
    let snapshot = match api::snapshot(uid).await {
      Ok(s) => s,
      Err(err) => {
        error!("Snapshot {} returned with error: {}: {}", uid, err.cause, err.desc);
        return;
      }
    };

    if let Err(err) = notifier::notify_users(&self.bot, &self.mongo, prev, &snapshot, silent).await {
      error!("An error occured while notifying users: {}", err);
    }
  }

  async fn wait(&self) -> bool {
    let now = utils::now(0);
    let is_night = now.time() < NaiveTime::from_hms_opt(6, 0, 0).unwrap();

    let wait = match !is_night {
      true => self
        .prev
        .next_update
        .signed_duration_since(now)
        .num_milliseconds()
        .clamp(1000 * 10, 1000 * 24 * 60 * 60),
      false => 6 * 60 * 60 - (now.timestamp() - utils::now_date(0).timestamp()),
    } as u64;

    info!("Sleeping for {}s in awaiting of next update", wait as f32 / 1000f32);
    sleep(Duration::from_millis(wait)).await;
    is_night
  }
}
