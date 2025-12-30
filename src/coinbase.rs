use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{
  fmt::{self},
  str::FromStr,
};

use anyhow::Result;
use serde::{Deserialize, Deserializer, de};
use tungstenite::Message as WebSocketMessage;
use tungstenite::WebSocket;
use tungstenite::stream::MaybeTlsStream;

use crate::book::Book;

pub struct CoinbaseClient {
  book: Book,
  ws:   WebSocket<MaybeTlsStream<TcpStream>>,
}

impl CoinbaseClient {
  pub fn new() -> Result<CoinbaseClient> {
    let (mut ws, _) = tungstenite::connect("wss://ws-feed.exchange.coinbase.com")?;

    ws.send(WebSocketMessage::text(format!(
      r#"{{"type": "subscribe", "product_ids": ["BTC-USD"], "channels": [ "ticker" ] }}"#,
    )))?;

    Ok(CoinbaseClient {
      book: Book::new(),
      ws,
    })
  }

  pub fn run(&mut self) -> Result<()> {
    loop {
      match self.ws.read()? {
        WebSocketMessage::Text(t) => {
          let msg: Message = serde_json::from_str(t.as_str()).unwrap();

          match msg {
            Message::Subscriptions => {}
            Message::Ticker { price, .. } => {
              println!("{}{}", termion::clear::All, price)
            }
          }
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
#[serde(tag = "type", rename_all = "snake_case")]
enum Message<'msg> {
  Subscriptions,
  Ticker {
    sequence: usize,
    #[serde(borrow)]
    price:    &'msg str,
    #[serde(borrow)]
    side:     &'msg str,
  },
}

// impl Display for Message {
//   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     match self {
//       Message::Subscriptions => todo!(),
//     }
//   }
// }

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
