use std::time;
use std::{env::args, time::UNIX_EPOCH};

use anyhow::Result;
use serde::Deserialize;
use tradyman::coinbase::CoinbaseClient;
use tradyman::polymarket::PolymarketClient;

fn main() -> Result<()> {
  // let market = args().nth(1).unwrap_or("btc-updown-15m".to_string());
  //
  // let unix_timestamp = time::SystemTime::now()
  //   .duration_since(UNIX_EPOCH)?
  //   .as_secs();
  // let unix_timestamp = unix_timestamp - unix_timestamp % 900 + 1800;
  //
  // let url = format!("https://gamma-api.polymarket.com/events/slug/{market}-{unix_timestamp}");
  // // println!("{url}");
  //
  // let event_info: PolymarketEventInfo = reqwest::blocking::get(url)?.json()?;
  //
  // // println!("{event_info}");
  //
  // let token_id = event_info.markets[0]
  //   .clob_token_ids
  //   .split("\"")
  //   .nth(1)
  //   .unwrap()
  //   .to_string();
  //
  // println!("{token_id}");
  //
  // let mut client = PolymarketClient::new(token_id)?;
  let mut client = CoinbaseClient::new()?;

  client.run()?;

  Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PolymarketEventInfo {
  markets: Vec<PolymarketMarketInfo>,
}

#[derive(Deserialize)]
struct PolymarketMarketInfo {
  #[serde(rename = "clobTokenIds")]
  clob_token_ids: String,
}

// fn get_token_id() -> String {
// }
