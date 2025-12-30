use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
  fmt::{self, Display},
  str::FromStr,
};

use anyhow::{Error, Result};
use serde::{Deserialize, Deserializer, de};
use termion::color;
use tungstenite::Message as WebSocketMessage;
use tungstenite::WebSocket;
use tungstenite::stream::MaybeTlsStream;

use crate::book::Book;
use crate::types::{Price, Side, Volume};

#[allow(dead_code)]
enum PolymarketSeries {
  // 15 min
  BitcoinUpdown15Min   = 10192,
  EthereumUpdown15Min  = 10191,
  SolanaUpdown15Min    = 10423,
  RippleUpdown15Min    = 10422,
  // 1h
  BitcoinUpdownHourly  = 10114,
  EthereumUpdownHourly = 10117,
  SolanaUpdownHourly   = 10122,
  RippleUpdownHourly   = 10123,
  // 4h
  BitcoinUpdown4h      = 10331,
  EthereumUpdown4h     = 10332,
  SolanaUpdown4h       = 10333,
  RippleUpdown4h       = 10327,
}

impl Display for PolymarketSeries {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PolymarketSeries::BitcoinUpdown15Min => "10192",
        PolymarketSeries::EthereumUpdown15Min => "10191",
        PolymarketSeries::SolanaUpdown15Min => "10423",
        PolymarketSeries::RippleUpdown15Min => "10422",
        PolymarketSeries::BitcoinUpdownHourly => "10114",
        PolymarketSeries::EthereumUpdownHourly => "10117",
        PolymarketSeries::SolanaUpdownHourly => "10122",
        PolymarketSeries::RippleUpdownHourly => "10123",
        PolymarketSeries::BitcoinUpdown4h => "10331",
        PolymarketSeries::EthereumUpdown4h => "10332",
        PolymarketSeries::SolanaUpdown4h => "10333",
        PolymarketSeries::RippleUpdown4h => "10327",
      }
    )
  }
}

pub struct PolymarketClient {
  token_id: String,
  book:     Book,
  ws:       WebSocket<MaybeTlsStream<TcpStream>>,
}

impl PolymarketClient {
  pub fn new(token_id: String) -> Result<PolymarketClient> {
    let (mut ws, _) = tungstenite::connect("wss://ws-subscriptions-clob.polymarket.com/ws/market")?;

    ws.send(WebSocketMessage::text(format!(
      r#"{{"assets_ids": ["{}"], "operation": "subscribe"}}"#,
      token_id.as_str()
    )))?;

    // Handle the first message because it's in a vec for some reason
    // Note that this is likely to cause some bugs cause what if we receive a normal
    // message (read: not wrapped in an array) before we get the response to our
    // subscription

    let mut book = Book::new();

    let WebSocketMessage::Text(t) = ws.read()? else {
      return Err(Error::msg(
        "First message from Polymarket after subscribe was not a text message",
      ));
    };

    let snapshot: Vec<Message> = serde_json::from_str(t.as_str()).unwrap();
    if let Message::Book { bids, asks } = &snapshot[0] {
      for bid in bids {
        book.set_one(Side::Buy, bid.price, bid.size);
      }
      for ask in asks {
        book.set_one(Side::Sell, ask.price, ask.size);
      }
    }

    Ok(PolymarketClient { token_id, book, ws })
  }

  pub fn run(&mut self) -> Result<()> {
    loop {
      match self.ws.read()? {
        WebSocketMessage::Text(t) => {
          let msg: Message = serde_json::from_str(t.as_str()).unwrap();

          match msg {
            Message::Book { bids, asks } => {
              self.book.set_all(bids, asks);
            }
            Message::PriceChange { price_changes, .. } => {
              for price_change in price_changes {
                if price_change.asset_id == self.token_id {
                  self
                    .book
                    .set_one(price_change.side, price_change.price, price_change.size);
                }
              }
            }

            Message::LastTradePrice { price } => self.book.set_last_traded_price(price),
            Message::TickSizeChange => {}
          }
          println!("{}{}", termion::clear::All, self.book);
        }
        _ => {
          eprintln!("=> got something other than text");
          break;
        }
      };
    }

    Ok(())
  }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
enum Message<'msg> {
  Book {
    bids: Vec<OrderSummary>,
    asks: Vec<OrderSummary>,
  },
  PriceChange {
    #[serde(borrow)]
    price_changes: Vec<PriceChange<'msg>>,
    // #[serde(deserialize_with = "deserialize_time")]
    // timestamp:     SystemTime,
  },
  LastTradePrice {
    price: Price,
  },
  TickSizeChange,
}

impl<'msg> Display for Message<'msg> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Message::Book { bids, asks } => write!(f, "book|{bids:?}|{asks:?}"),
      Message::PriceChange { price_changes, .. } => write!(f, "price_change|{price_changes:?}"),
      Message::LastTradePrice { price } => write!(f, "last_trade_price,{:.2}", price.0),
      Message::TickSizeChange => write!(f, "tick_size_change"),
    }
  }
}

#[allow(dead_code)]
fn deserialize_time<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SystemTime, D::Error> {
  struct MyVisitor;

  impl<'de> de::Visitor<'de> for MyVisitor {
    type Value = SystemTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("a string containing json data")
    }

    fn visit_str<E>(self, value: &str) -> Result<SystemTime, E>
    where
      E: de::Error,
    {
      let millis = u64::from_str(value).unwrap();

      Ok(UNIX_EPOCH + Duration::from_millis(millis))
    }
  }

  deserializer.deserialize_str(MyVisitor)
}

#[derive(Debug, Deserialize)]
pub struct OrderSummary {
  pub price: Price,
  pub size:  Volume,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PriceChange<'msg> {
  pub asset_id: &'msg str,
  pub price:    Price,
  pub size:     Volume,
  pub side:     Side,
}

impl<'msg> Display for PriceChange<'msg> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.side {
      Side::Buy => write!(
        f,
        "{}BID{}   :: {} shares @ {}",
        color::Fg(color::Green),
        color::Fg(color::Reset),
        self.size.0,
        self.price.0
      ),
      Side::Sell => write!(
        f,
        "{}OFFER{} :: {} shares @ {}",
        color::Fg(color::Red),
        color::Fg(color::Reset),
        self.size.0,
        self.price.0
      ),
    }
  }
}

impl Display for Side {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Buy => "BUY",
        Self::Sell => "SELL",
      }
    )
  }
}
