use {
  bitcoincore_rpc::{Auth, Client, RpcApi},
  std::{
    fs,
    net::TcpListener,
    path::Path,
    process::{Child, Command},
    thread,
    time::Duration,
  },
};

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

  let client = Client::new(
    &format!("http://localhost:{port}"),
    Auth::CookieFile(cookie_file.into()),
  )
  .unwrap();

  for attempt in 0..=300 {
    match client.get_blockchain_info() {
      Ok(_) => break,
      Err(err) => {
        if attempt == 300 {
          panic!("Failed to connect to bitcoind: {err}");
        }
      }
    }
    thread::sleep(Duration::from_millis(100));
  }

  for i in 0..10000 {
    eprintln!("getting block {i}…");
    let hash = client.get_block_hash(0).unwrap();
    client.get_block(&hash).unwrap();
  }

  drop(child);
}
