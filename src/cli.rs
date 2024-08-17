use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::{builder::Styles, Parser};
use clap_verbosity_flag::{InfoLevel, Verbosity};

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
  #[clap(flatten)]
  pub verbose: Verbosity<InfoLevel>,

  /// The name of the Kubernetes pod
  #[clap(short, long)]
  pub pod: String,

  /// The namespace of the Kubernetes pod
  #[clap(short, long, default_value = "default")]
  pub namespace: String,
}

impl Cli {
  pub fn collect(&self) -> Result<()> {
    println!(
      "Collecting Kubernetes pod event time durations for the pod `{}` in the namespace `{}`",
      self.pod, self.namespace
    );

    Ok(())
  }
}
