use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
  fmt::{self, Display},
  str::FromStr,
};

use serde::{Deserialize, Deserializer, de};
use termion::color;
use tungstenite::{
  Message as WebSocketMessage,
  WebSocket,
  error::Error as WebSocketError,
  stream::MaybeTlsStream,
};

use crate::book::Book;
use crate::ringbuffer::RingBuffer;
use crate::types::{Price, Volume};

pub struct PolymarketClient {
  token_id:              String,
  book:                  Book,
  ws:                    WebSocket<MaybeTlsStream<TcpStream>>,
  latency_ringbuffer:    RingBuffer<32>,
  throughput_ringbuffer: RingBuffer<32>,
}

impl PolymarketClient {
  pub fn new(token_id: String) -> Result<PolymarketClient, WebSocketError> {
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

    if let WebSocketMessage::Text(t) = ws.read()? {
      let snapshot: Vec<Message> = serde_json::from_str(t.as_str()).unwrap();
      if let Message::Book { bids, asks } = &snapshot[0] {
        for bid in bids {
          book.set(Side::Buy, bid.price, bid.size);
        }
        for ask in asks {
          book.set(Side::Sell, ask.price, ask.size);
        }
      }
    }

    Ok(PolymarketClient {
      token_id,
      book,
      ws,
      latency_ringbuffer: RingBuffer::new(),
      throughput_ringbuffer: RingBuffer::new(),
    })
  }

  pub fn run(&mut self) -> Result<(), WebSocketError> {
    loop {
      match self.ws.read()? {
        WebSocketMessage::Text(t) => {
          let msg: Message = serde_json::from_str(t.as_str()).unwrap();
          if let Message::PriceChange {
            price_changes,
            timestamp,
          } = msg
          {
            let latency = SystemTime::now()
              .duration_since(timestamp)
              .unwrap()
              .as_millis() as usize;

            self.latency_ringbuffer.push(latency);
            for price_change in price_changes {
              if price_change.asset_id == self.token_id {
                self
                  .book
                  .set(price_change.side, price_change.price, price_change.size);
              }
            }
          }
          println!("{}{}", termion::clear::All, self.book);
          println!("{}", self.latency_ringbuffer.average());
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
    #[serde(deserialize_with = "deserialize_time")]
    timestamp:     SystemTime,
  },
  LastTradePrice,
  TickSizeChange,
}

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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
  Buy,
  Sell,
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
