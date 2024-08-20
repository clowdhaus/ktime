use std::{collections::HashMap, path::PathBuf};

use anstyle::{AnsiColor, Color, Style};
use anyhow::{bail, Context, Result};
use clap::{builder::Styles, Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use k8s_openapi::api::core::v1::Pod;
use kube::{
  api::{Api, DynamicObject, Patch, PatchParams, ResourceExt},
  core::GroupVersionKind,
  discovery::Discovery,
};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use tracing::{info, warn};

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
  Collect(CollectInput),
  #[command(arg_required_else_help = true)]
  Run(RunInput),
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

/// Apply the Kubernetes manifest file and collect the pod event time durations
#[derive(Args, Debug, Serialize, Deserialize)]
pub struct RunInput {
  /// The Kubernetes context to use
  #[clap(short, long)]
  pub path: PathBuf,
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

pub async fn collect(input: &CollectInput, client: kube::Client) -> Result<()> {
  let pods: Api<Pod> = Api::namespaced(client, &input.namespace);
  let pod = match pods.get(&input.name).await {
    Ok(p) => p,
    Err(_) => bail!("Pod `{}` not found in namespace `{}`", input.name, input.namespace),
  };

  let status = pod.status.unwrap();

  while *status.phase.as_ref().unwrap() != "Running" {
    tokio::time::sleep(Duration::from_secs(15)).await;
  }

  let mut conditions: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();
  for cond in status.conditions.unwrap() {
    let ltt = cond.last_transition_time.unwrap().0;

    conditions.insert(cond.type_, ltt);
  }

  get_pod_status_timings(conditions).await?;

  Ok(())
}

pub async fn run(input: &RunInput, client: kube::Client) -> Result<()> {
  let discovery = Discovery::new(client.clone()).run().await?;
  let ssapply = PatchParams::apply("ktime").force();
  let yaml =
    std::fs::read_to_string(&input.path).with_context(|| format!("Failed to read {}", input.path.display()))?;

  let de = serde_yaml::Deserializer::from_str(&yaml);
  let doc = serde_yaml::Value::deserialize(de)?;

  let obj: DynamicObject = serde_yaml::from_value(doc)?;
  let namespace = obj.metadata.namespace.as_deref().or(Some("default"));
  let gvk = if let Some(tm) = &obj.types {
    GroupVersionKind::try_from(tm)?
  } else {
    bail!("cannot apply object without valid TypeMeta {:?}", obj);
  };

  let name = obj.name_any();
  if let Some((ar, _caps)) = discovery.resolve_gvk(&gvk) {
    let api: Api<DynamicObject> = match namespace {
      Some(namespace) => Api::namespaced_with(client, namespace, &ar),
      None => Api::default_namespaced_with(client, &ar),
    };

    info!("Applying {}: \n{}", gvk.kind, serde_yaml::to_string(&obj)?);
    let data: serde_json::Value = serde_json::to_value(&obj)?;
    let _r = api.patch(&name, &ssapply, &Patch::Apply(data)).await?;
    info!("Applied {} `{}`", gvk.kind, name);
  } else {
    warn!("Cannot apply document for unknown {:?}", gvk);
  }

  Ok(())
}
