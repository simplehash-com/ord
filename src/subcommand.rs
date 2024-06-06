use super::*;

pub mod balances;
pub mod decode;
pub mod env;
pub mod epochs;
pub mod find;
pub mod index;
pub mod list;
pub mod parse;
pub mod runes;
pub(crate) mod server;
mod settings;
pub mod subsidy;
pub mod supply;
pub mod teleburn;
pub mod traits;
pub mod wallet;

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "List all rune balances")]
  Balances,
  #[command(about = "Decode a transaction")]
  Decode(decode::Decode),
  #[command(about = "Start a regtest ord and bitcoind instance")]
  Env(env::Env),
  #[command(about = "List the first satoshis of each reward epoch")]
  Epochs,
  #[command(about = "Find a satoshi's current location")]
  Find(find::Find),
  #[command(subcommand, about = "Index commands")]
  Index(index::IndexSubcommand),
  #[command(about = "List the satoshis in an output")]
  List(list::List),
  #[command(about = "Parse a satoshi from ordinal notation")]
  Parse(parse::Parse),
  #[command(about = "List all runes")]
  Runes,
  #[command(about = "Run the explorer server")]
  Server(server::Server),
  #[command(about = "Display settings")]
  Settings,
  #[command(about = "Display information about a block's subsidy")]
  Subsidy(subsidy::Subsidy),
  #[command(about = "Display Bitcoin supply information")]
  Supply,
  #[command(about = "Generate teleburn addresses")]
  Teleburn(teleburn::Teleburn),
  #[command(about = "Display satoshi traits")]
  Traits(traits::Traits),
  #[command(about = "Wallet commands")]
  Wallet(wallet::WalletCommand),
}

impl Subcommand {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self {
      Self::Balances => balances::run(settings),
      Self::Decode(decode) => decode.run(settings),
      Self::Env(env) => env.run(),
      Self::Epochs => epochs::run(),
      Self::Find(find) => find.run(settings),
      Self::Index(index) => index.run(settings),
      Self::List(list) => list.run(settings),
      Self::Parse(parse) => parse.run(),
      Self::Runes => runes::run(settings),
      Self::Server(server) => {
        if settings.emit_events() {
          log::info!("Starting server with kafka event emitter...");
          let (sender, mut receiver) = tokio::sync::mpsc::channel::<Event>(128);
          let index = Arc::new(Index::open_with_event_sender(&settings, Some(sender))?);
          let handle = axum_server::Handle::new();
          LISTENERS.lock().unwrap().push(handle.clone());
          thread::spawn(move || {
            let stream = stream::StreamClient::new();
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
              while let Some(event) = receiver.recv().await {
                loop {
                  match stream.emit(&event) {
                    Err(err) => {
                      log::error!("Error emitting kafka event: {:?}. Sleeping for 1 sec", err);
                      tokio::time::sleep(Duration::from_secs(1)).await;
                      continue;
                    }
                    Ok(_) => break,
                  }
                }
              }
            });
            stream.flush()
          });
          server.run(settings, index, handle)
        } else {
          let index = Arc::new(Index::open(&settings)?);
          let handle = axum_server::Handle::new();
          LISTENERS.lock().unwrap().push(handle.clone());
          server.run(settings, index, handle)
        }
      }
      Self::Settings => settings::run(settings),
      Self::Subsidy(subsidy) => subsidy.run(),
      Self::Supply => supply::run(),
      Self::Teleburn(teleburn) => teleburn.run(),
      Self::Traits(traits) => traits.run(),
      Self::Wallet(wallet) => wallet.run(settings),
    }
  }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum OutputFormat {
  #[default]
  Json,
  Yaml,
  Minify,
}

pub trait Output: Send {
  fn print(&self, format: OutputFormat);
}

impl<T> Output for T
where
  T: Serialize + Send,
{
  fn print(&self, format: OutputFormat) {
    match format {
      OutputFormat::Json => serde_json::to_writer_pretty(io::stdout(), self).ok(),
      OutputFormat::Yaml => serde_yaml::to_writer(io::stdout(), self).ok(),
      OutputFormat::Minify => serde_json::to_writer(io::stdout(), self).ok(),
    };
    println!();
  }
}

pub(crate) type SubcommandResult = Result<Option<Box<dyn Output>>>;
