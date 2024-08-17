use anyhow::Result;
use clap::Parser;
use tracing_log::AsTrace;
use tracing_subscriber::FmtSubscriber;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<()> {
  let cli = ktime::Cli::parse();
  let subscriber = FmtSubscriber::builder()
    .with_max_level(cli.verbose.log_level_filter().as_trace())
    .without_time()
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

  match cli.commands {
    ktime::Commands::Collect(args) => ktime::collect(&args).await?,
    ktime::Commands::Run(args) => ktime::run(&args).await?,
  }

  Ok(())
}
