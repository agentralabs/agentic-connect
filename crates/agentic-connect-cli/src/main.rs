//! AgenticConnect CLI — command-line interface for connection management.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "acnx", about = "AgenticConnect — universal external interface")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Test connectivity to a target
    Ping {
        /// URL or host:port to test
        target: String,
    },
    /// List configured connections
    List,
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Ping { target } => {
            println!("Testing connectivity to: {}", target);
            // TODO: implement via agentic_connect engine
        }
        Command::List => {
            println!("No connections configured yet.");
        }
        Command::Version => {
            println!("acnx {}", env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}
