use std::{collections::BTreeMap, fmt::Display};

use termion::color;

use crate::{
  polymarket::OrderSummary,
  types::{Price, Side, Volume},
};

pub struct Book {
  buy_side:          BTreeMap<Price, Volume>,
  sell_side:         BTreeMap<Price, Volume>,
  last_traded_price: Price,
}

impl Default for Book {
  fn default() -> Self {
    Self::new()
  }
}

impl Book {
  pub fn new() -> Book {
    Book {
      sell_side:         BTreeMap::new(),
      buy_side:          BTreeMap::new(),
      last_traded_price: Price(-1.0),
    }
  }

  pub fn set_last_traded_price(&mut self, last_traded_price: Price) {
    self.last_traded_price = last_traded_price;
  }

  pub fn set_all(&mut self, bids: Vec<OrderSummary>, asks: Vec<OrderSummary>) {
    self.buy_side.clear();
    self.sell_side.clear();

    for bid in bids {
      self.buy_side.insert(bid.price, bid.size);
    }
    for ask in asks {
      self.sell_side.insert(ask.price, ask.size);
    }
  }

  pub fn set_one(&mut self, side: Side, price: Price, volume: Volume) {
    let side = match side {
      Side::Buy => &mut self.buy_side,
      Side::Sell => &mut self.sell_side,
    };

    if volume.0 == 0.0 {
      side.remove(&price);
    } else {
      side.insert(price, volume);
    }
  }
}

const BAR: &str = "############################################################";

impl Display for Book {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let n = 10;
    let max_volume = self
      .sell_side
      .iter()
      .take(n)
      .chain(self.buy_side.iter().rev().take(n))
      .max_by(|(_, a), (_, b)| a.0.total_cmp(&b.0))
      .unwrap()
      .1
      .0;

    let pad = 1 + max_volume.log10() as usize + 3;

    for entry in self.sell_side.iter().take(n).rev() {
      writeln!(
        f,
        "{:.4} {:<pad$} {}{}{}",
        entry.0.0,
        entry.1.0,
        color::Fg(color::Red),
        &BAR[..(BAR.len() * entry.1.0 as usize).div_ceil(max_volume as usize)],
        color::Fg(color::Reset),
      )?
    }
    writeln!(f, "Last traded price {:.2}", self.last_traded_price.0)?;
    for entry in self.buy_side.iter().rev().take(n) {
      writeln!(
        f,
        "{:.4} {:<pad$} {}{}{}",
        entry.0.0,
        entry.1.0,
        color::Fg(color::Green),
        &BAR[..(BAR.len() * entry.1.0 as usize).div_ceil(max_volume as usize)],
        color::Fg(color::Reset),
      )?
    }

    Ok(())
  }
}
