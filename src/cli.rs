use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Run the secure web interface (for offline device interaction)
    Web {
        /// Optional port number (default: random high port)
        #[arg(short, long)]
        port: Option<u16>,
        /// IP address to bind to (default: 127.0.0.1)
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,
        /// GPG Home directory override
        #[arg(long)]
        gpg_dir: Option<String>,
    },
    // Add other direct CLI commands later if needed
    // KeyGen { ... },
    // Encrypt { ... },
}