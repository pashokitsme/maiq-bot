use std::time::Duration;

use chrono::{DateTime, NaiveTime, Utc};
use maiq_api_wrapper::{api, polling::SnapshotChanges};
use maiq_shared::utils::time::*;
use teloxide::Bot;
use tokio::time::sleep;

use crate::{bot::notifier, db::MongoPool};

pub struct Poller {
  bot: Bot,
  mongo: MongoPool,
}

impl Poller {
  pub async fn new(bot: Bot, mongo: MongoPool) -> Self {
    Self { bot, mongo }
  }

  pub async fn run(&mut self) {
    loop {
      if now().time() < NaiveTime::from_hms_opt(6, 0, 0).unwrap() {
        let wait_s = 7 * 60 * 60 - (now().timestamp() - now_date().timestamp()) as u64;
        info!("Sleeping due to the night for {}s", wait_s);
        sleep(Duration::from_secs(wait_s)).await;
      };

      let poll = match api::poll().await {
        Ok(p) => p,
        Err(err) => {
          error!("Couldn't make a poll request: {}: {}", err.cause, err.desc);
          sleep(Duration::from_secs(10)).await;
          continue;
        }
      };

      self.notify_if_need(&poll.today).await;
      self.notify_if_need(&poll.next).await;
      self.wait(poll.next_update).await;
    }
  }

  async fn notify_if_need(&self, new: &SnapshotChanges) {
    if new.groups.is_empty() || new.uid.is_none() {
      return;
    }

    self.notify(&new.uid.as_ref().unwrap(), new).await
  }

  async fn notify(&self, uid: &str, changes: &SnapshotChanges) {
    let snapshot = match api::snapshot(uid).await {
      Ok(s) => s,
      Err(err) => {
        error!("Snapshot {} returned with error: {}: {}", uid, err.cause, err.desc);
        return;
      }
    };

    if let Err(err) = notifier::notify_update(&self.bot, &self.mongo, changes, snapshot).await {
      error!("An error occured while notifying users: {}", err);
    }
  }

  async fn wait(&self, next_update: DateTime<Utc>) {
    let wait = next_update
      .signed_duration_since(now())
      .num_milliseconds()
      .clamp(1000 * 10, 1000 * 24 * 60 * 60) as u64;

    info!("Sleeping for {}s in awaiting of next update", wait as f32 / 1000f32);
    sleep(Duration::from_millis(wait)).await;
  }
}
