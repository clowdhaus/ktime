use anyhow::{Result, bail};
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

  let k8s_client = match kube::Client::try_default().await {
    Ok(client) => client,
    Err(_) => {
      bail!("Unable to connect to cluster. Ensure kubeconfig file is present and updated to connect to the cluster.");
    }
  };

  match cli.commands {
    ktime::Commands::Apply(args) => ktime::apply(&args, k8s_client).await?,
    ktime::Commands::Collect(args) => ktime::collect(&args, k8s_client).await?,
  }

  Ok(())
}
