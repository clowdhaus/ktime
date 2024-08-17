use anstyle::{AnsiColor, Color, Style};
use anyhow::{bail, Result};
use clap::{builder::Styles, Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
  Collect(Pod),
  #[command(arg_required_else_help = true)]
  Run(Pod),
}

/// Analyze an Amazon EKS cluster for potential upgrade issues
#[derive(Args, Debug, Serialize, Deserialize)]
pub struct Pod {
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

pub async fn collect(pod: &Pod) -> Result<()> {
  println!(
    "Collecting Kubernetes pod event time durations for the pod `{}` in the namespace `{}`",
    pod.name, pod.namespace
  );

  Ok(())
}

pub async fn run(pod: &Pod) -> Result<()> {
  let path = match &pod.path {
    Some(p) => p.display().to_string(),
    None => bail!("The path to the manifest file is required"),
  };

  println!(
    "Applying the manifest at `{path}` and collecting Kubernetes pod event time durations for the pod `{}` in the namespace `{}`",
    pod.name, pod.namespace
  );

  Ok(())
}
