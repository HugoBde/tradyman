use std::{collections::BTreeMap, fmt::Display};

use termion::color;

use crate::{
  polymarket::Side,
  types::{Price, Volume},
};

pub struct Book {
  buy_side:  BTreeMap<Price, Volume>,
  sell_side: BTreeMap<Price, Volume>,
}

impl Default for Book {
  fn default() -> Self {
    Self::new()
  }
}

impl Book {
  pub fn new() -> Book {
    Book {
      sell_side: BTreeMap::new(),
      buy_side:  BTreeMap::new(),
    }
  }

  pub fn set(&mut self, side: Side, price: Price, volume: Volume) {
    let side = match side {
      Side::Buy => &mut self.buy_side,
      Side::Sell => &mut self.sell_side,
    };

    if volume.0 == 0 {
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
      .max_by_key(|(_, size)| size.0)
      .unwrap()
      .1
      .0;

    let pad = 1 + max_volume.ilog10() as usize;

    for entry in self.sell_side.iter().take(n).rev() {
      writeln!(
        f,
        "{:.4} {:<pad$} {}{}{}",
        entry.0.0,
        entry.1.0,
        color::Fg(color::Red),
        &BAR[..(BAR.len() * entry.1.0).div_ceil(max_volume)],
        color::Fg(color::Reset),
      )?
    }
    writeln!(f)?;
    for entry in self.buy_side.iter().rev().take(n) {
      writeln!(
        f,
        "{:.4} {:<pad$} {}{}{}",
        entry.0.0,
        entry.1.0,
        color::Fg(color::Green),
        &BAR[..(BAR.len() * entry.1.0).div_ceil(max_volume)],
        color::Fg(color::Reset),
      )?
    }

    Ok(())
  }
}
