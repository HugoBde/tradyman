use std::env::args;

use tradyman::polymarket::PolymarketClient;
use tungstenite::Error as WebSocketError;

fn main() -> Result<(), WebSocketError> {
  let token = args().nth(1).unwrap_or(
    "54533043819946592547517511176940999955633860128497669742211153063842200957669".to_string(),
  );
  let mut client = PolymarketClient::new(token).unwrap();

  client.run(10_000)?;

  Ok(())
}
