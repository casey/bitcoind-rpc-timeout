use std::{
  fs,
  net::TcpListener,
  path::Path,
  process::{Child, Command},
  thread,
  time::Duration,
};

use jsonrpc::Client;

use serde_json::value::RawValue;

struct Kill(Child);

impl Drop for Kill {
  fn drop(&mut self) {
    self.0.kill().unwrap();
  }
}

fn main() {
  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  fs::remove_dir_all("regtest").ok();

  let child = Kill(
    Command::new("bitcoind")
      .args(&["-regtest", "-datadir=.", &format!("-rpcport={port}")])
      .spawn()
      .unwrap(),
  );

  let cookie_file = Path::new("regtest/.cookie");

  while !cookie_file.exists() {
    eprintln!("Waiting for cookie file…");
    thread::sleep(Duration::from_millis(100));
  }

  let cookie = fs::read_to_string(cookie_file).unwrap();

  let (user, pass) = cookie.split_once(':').unwrap();

  let client = Client::simple_http(
    &format!("http://localhost:{port}"),
    Some(user.into()),
    Some(pass.into()),
  )
  .unwrap();

  for attempt in 0..=300 {
    let request = client.build_request("getblockchaininfo", &[]);
    match client.send_request(request) {
      Ok(_) => break,
      Err(err) => {
        if attempt == 300 {
          panic!("Failed to connect to bitcoind: {err}");
        }
      }
    }
    thread::sleep(Duration::from_millis(100));
  }

  let hash = RawValue::from_string(
    "\"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f\"".into(),
  )
  .unwrap();

  for i in 0..20000 {
    eprintln!("getting block {i}…");
    let args = &[hash.clone()];
    let request = client.build_request("getblock", args);
    client.send_request(request).unwrap();
  }

  drop(child);
}
