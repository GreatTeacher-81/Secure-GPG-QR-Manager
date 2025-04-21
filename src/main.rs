mod cli;
mod error;
mod gpg_ops;
mod qr_utils;
mod web_handlers;
mod web_server;

use cli::{CliArgs, Commands};
use clap::Parser;
use error::Result; // Use custom result type

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    match args.command {
        Commands::Web { port, bind, gpg_dir } => {
             // Validate bind address format early
             if let Err(_) = bind.parse::<std::net::IpAddr>() {
                  eprintln!("Error: Invalid IP address format for --bind: {}", bind);
                  std::process::exit(1);
             }
             println!("Starting web server mode...");
             web_server::run_web_server(bind, port, gpg_dir).await?;
        }
        // Add handlers for other CLI commands if implemented
    }

    Ok(())
}
