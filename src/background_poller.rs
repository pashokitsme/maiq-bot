use std::time::Duration;

use maiq_shared::utils;
use teloxide::Bot;
use tokio::time::sleep;

use crate::api::{self, Poll};

pub struct Poller {
  bot: Bot,
  prev: Poll,
}

impl Poller {
  pub async fn new(bot: Bot) -> Self {
    let first_poll = api::poll().await.expect("Couldn't make a first poll");
    let svc = Self { bot, prev: first_poll };
    svc
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

      if self.prev.latest_today_uid != poll.latest_today_uid {
        todo!();
      }

      if self.prev.latest_next_uid != poll.latest_next_uid {
        todo!();
      }

      self.prev = poll;
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
