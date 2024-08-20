# ktime

Collect Kubernetes pod event time durations

## Usage

```sh
ktime - Collect Kubernetes pod event time durations

Usage: ktime [OPTIONS] <COMMAND>

Commands:
  apply    Apply the Kubernetes manifest file containing the pod definition and collect the pod event time durations
  collect  Collect the pod event time durations of an existing Kubernetes pod
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
  -V, --version     Print version
```

Collect the pod event time durations of an existing Kubernetes pod:

```sh
ktime collect --pod my-pod # default namespace
ktime collect --pod my-pod --namespace my-namespace
```

Apply the Kubernetes manifest file containing the pod definition and collect the pod event time durations:

```sh
ktime apply --file my-pod.yaml
```

<!-- <p align="center">
  <img src=".github/demo.svg" alt="ktime demo">
</p> -->

## Installation

[Archives of pre-compiled binaries for `ktime` are available for Windows, macOS and Linux.](https://github.com/clowdhaus/ktime/releases)

### Homebrew (macOS and Linux)

```sh
brew install clowdhaus/taps/ktime
```

### Cargo (rust)

```sh
cargo install ktime
```

### Source

`ktime` is written in Rust, so you'll need to grab a [Rust installation](https://www.rust-lang.org/) in order to compile it.
`ktime` compiles with Rust 1.79.0 (stable) or newer. In general, `ktime` tracks the latest stable release of the Rust compiler.

To build `ktime`:

```sh
git clone https://github.com/clowdhaus/ktime
cd ktime
cargo build --release
./target/release/ktime --version
0.1.0
```

## Local Development

`ktime` uses Rust stable for production builds, but nightly for local development for formatting and linting. It is not a requirement to use nightly, but if running `fmt` you may see a few warnings on certain features only being available on nightly.

Build the project to pull down dependencies and ensure everything is setup properly:

```sh
cargo build
```

To format the codebase:

If using nightly to use features defined in [rustfmt.toml](rustfmt.toml), run the following:

```sh
cargo +nightly fmt --all
```

If using stable, run the following:

```sh
cargo fmt --all
```

To execute lint checks:

```sh
cargo clippy --all-targets --all-features
```

To run `ktime` locally for development:

```sh
cargo run
```

### Running Tests

To execute the tests provided, run the following from the project root directory:

```sh
cargo test --all
```
