use std::{collections::HashMap, path::PathBuf};

use anstyle::{AnsiColor, Color, Style};
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, builder::Styles};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use futures::prelude::*;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, PostParams, ResourceExt, WatchEvent, WatchParams};
use serde::{Deserialize, Serialize};
use tracing::info;

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
  Apply(ApplyInput),

  #[command(arg_required_else_help = true)]
  Collect(CollectInput),
}

/// Collect the pod event time durations of an existing Kubernetes pod
#[derive(Args, Debug, Serialize, Deserialize)]
pub struct CollectInput {
  /// The name of the Kubernetes pod
  #[clap(short, long)]
  pub name: String,

  /// The namespace of the Kubernetes pod
  #[clap(alias = "ns", long, default_value = "default")]
  pub namespace: String,
}

/// Apply the Kubernetes manifest file containing the pod definition and collect the pod event time durations
#[derive(Args, Debug, Serialize, Deserialize)]
pub struct ApplyInput {
  /// The path to the Kubernetes manifest file to apply
  #[clap(short, long)]
  pub file: PathBuf,
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

/// Watch for pod to be in the `Running` phase
async fn is_pod_running(name: &str, pod_api: Api<Pod>) -> Result<bool> {
  let pod = pod_api.get(name).await?;
  let status = pod.status.unwrap();

  // Exit early if the pod is already in the `Running` phase
  if status.phase.unwrap() == "Running" {
    return Ok(true);
  }

  let wp = WatchParams::default()
    .timeout(30)
    .fields(&format!("metadata.name={name}"));

  let mut stream = pod_api.watch(&wp, "0").await?.boxed();
  while let Some(status) = stream.try_next().await? {
    match status {
      WatchEvent::Modified(s) => {
        let phase = s.status.as_ref().unwrap().phase.as_ref().unwrap();
        info!("Pod `{}` => {phase}", s.name_any());

        if phase == "Running" {
          return Ok(true);
        }
      }
      WatchEvent::Error(s) => println!("{s}"),
      _ => (),
    }
  }

  let pod = pod_api.get(name).await?;
  let status = pod.status.unwrap();

  Ok(status.phase.unwrap() == "Running")
}

/// Get the pod conditions from the pod status
fn get_conditions(pod: Pod) -> Conditions {
  let status = pod.status.unwrap();

  let mut conditions: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();
  for cond in status.conditions.unwrap() {
    let ltt = cond.last_transition_time.unwrap().0;

    conditions.insert(cond.type_, ltt);
  }

  conditions
}

/// Collect the pod event time durations of an existing Kubernetes pod
pub async fn collect(input: &CollectInput, client: kube::Client) -> Result<()> {
  let pod_api: Api<Pod> = Api::namespaced(client.clone(), &input.namespace);

  loop {
    if is_pod_running(&input.name, pod_api.clone()).await? {
      break;
    }
    info!("Waiting for pod `{}` to be in the `Running` phase...", &input.name);
  }

  let pod = pod_api.get(&input.name).await?;
  let conditions = get_conditions(pod);

  get_pod_status_timings(conditions).await
}

/// Apply the Kubernetes manifest file and collect the pod event time durations
pub async fn apply(input: &ApplyInput, client: kube::Client) -> Result<()> {
  let manifest =
    std::fs::read_to_string(&input.file).with_context(|| format!("Failed to read `{}`", input.file.display()))?;

  let de = serde_yaml::Deserializer::from_str(&manifest);
  let pod_doc = serde_yaml::Value::deserialize(de)?;
  let pod: Pod = match serde_yaml::from_value(pod_doc) {
    Ok(p) => p,
    Err(e) => bail!("Failed to deserialize pod object: {e}\n Only pod manifests are supported at this time."),
  };

  let mut namespace = "default";

  // Only pods are supported
  let pod_api: Api<Pod> = match pod.metadata.namespace.as_deref() {
    Some(ns) => {
      namespace = ns;
      Api::namespaced(client.clone(), ns)
    }
    None => Api::default_namespaced(client.clone()),
  };

  let name = pod.name_any();

  let pp = PostParams::default();
  let _pod = pod_api.create(&pp, &pod).await?;
  info!("Pod `{name}` applied");

  collect(
    &CollectInput {
      name,
      namespace: namespace.to_string(),
    },
    client,
  )
  .await
}
