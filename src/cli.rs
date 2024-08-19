use std::{collections::HashMap, path::PathBuf};

use anstyle::{AnsiColor, Color, Style};
use anyhow::{bail, Result};
use clap::{builder::Styles, Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

/// Styles for CLI
fn get_styles() -> Styles {
  Styles::styled()
    .header(
      Style::new()
        .bold()
        .underline()
        .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
    )
    .literal(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
    .usage(
      Style::new()
        .bold()
        .underline()
        .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
    )
    .placeholder(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Magenta))))
}

/// ktime - Collect Kubernetes pod event time durations
#[derive(Debug, Parser)]
#[command(author, about, version)]
#[command(propagate_version = true)]
#[command(styles=get_styles())]
pub struct Cli {
  #[command(subcommand)]
  pub commands: Commands,

  #[clap(flatten)]
  pub verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
  #[command(arg_required_else_help = true)]
  Collect(Input),
  #[command(arg_required_else_help = true)]
  Run(Input),
}

/// Analyze an Amazon EKS cluster for potential upgrade issues
#[derive(Args, Debug, Serialize, Deserialize)]
pub struct Input {
  /// The name of the Kubernetes pod
  #[clap(short, long)]
  pub name: String,

  /// The namespace of the Kubernetes pod
  #[clap(alias = "ns", long, default_value = "default")]
  pub namespace: String,

  /// The Kubernetes context to use
  #[clap(short, long)]
  pub path: Option<PathBuf>,
}

type Conditions = HashMap<String, chrono::DateTime<chrono::Utc>>;

async fn get_pod_status_timings(conditions: Conditions) -> Result<()> {
  let pod_scheduled = conditions.get("PodScheduled").unwrap();

  if let Some(initialized) = conditions.get("Initialized") {
    println!("Initialize: {:?}s", (*initialized - *pod_scheduled).num_seconds());
  }

  if let Some(pod_ready_to_start_containers) = conditions.get("PodReadyToStartContainers") {
    println!(
      "Pod ready to start containers: {:?}s",
      (*pod_ready_to_start_containers - *pod_scheduled).num_seconds()
    );
  }

  if let Some(containers_ready) = conditions.get("ContainersReady") {
    println!(
      "Containers ready: {:?}s",
      (*containers_ready - *pod_scheduled).num_seconds()
    );
  }

  if let Some(ready) = conditions.get("Ready") {
    println!("Ready: {:?}s", (*ready - *pod_scheduled).num_seconds());
  }

  Ok(())
}

pub async fn collect(input: &Input, client: kube::Client) -> Result<()> {
  let pods: Api<Pod> = Api::namespaced(client, &input.namespace);
  let pod = match pods.get(&input.name).await {
    Ok(p) => p,
    Err(_) => bail!("Pod `{}` not found in namespace `{}`", input.name, input.namespace),
  };

  let status = pod.status.unwrap();

  while *status.phase.as_ref().unwrap() != "Running" {
    tokio::time::sleep(Duration::from_secs(15)).await;
  }

  println!("{:#?}", status.conditions);

  let mut conditions: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();
  for cond in status.conditions.unwrap() {
    let ltt = cond.last_transition_time.unwrap().0;

    conditions.insert(cond.type_, ltt);
  }

  get_pod_status_timings(conditions).await?;

  Ok(())
}

pub async fn run(input: &Input, _client: kube::Client) -> Result<()> {
  let path = match &input.path {
    Some(p) => p.display().to_string(),
    None => bail!("The path to the manifest file is required"),
  };

  println!(
    "Applying the manifest at `{path}` and collecting Kubernetes pod event time durations for the pod `{}` in the namespace `{}`",
    input.name, input.namespace
  );

  Ok(())
}
