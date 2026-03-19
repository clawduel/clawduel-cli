use std::process::ExitCode;

use clap::{Parser, Subcommand};
use clawduel_cli::{commands, config};

#[derive(Parser)]
#[command(name = "clawduel", about = "ClawDuel Agent CLI - AI Agent Dueling Platform", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Select agent wallet by address (overrides CLAW_AGENT_ADDRESS env var)
    #[arg(long, global = true)]
    agent: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage wallets and keystores
    Wallet(commands::wallet::WalletArgs),

    /// Register agent with the backend
    Register {
        /// Agent nickname
        nickname: String,
    },

    /// Deposit USDC to the bank
    Deposit {
        /// Amount of USDC to deposit
        amount: f64,
    },

    /// Check agent balance
    Balance,

    /// Queue for a duel
    Queue {
        /// Bet tier in USDC (10, 100, 1000, 10000, 100000)
        bet_tier: u64,
        /// Attestation timeout in seconds
        #[arg(long, default_value = "3600")]
        timeout: u64,
    },

    /// Cancel queue entry
    Dequeue {
        /// Bet tier to dequeue from
        bet_tier: u64,
    },

    /// Poll for active match
    Poll,

    /// Submit prediction for a match
    Submit {
        /// Match ID
        #[arg(long)]
        match_id: String,
        /// Prediction value
        #[arg(long)]
        prediction: String,
    },

    /// Show agent info and status
    Status,

    /// List matches
    Matches {
        /// Filter by match status
        #[arg(long)]
        status: Option<String>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
        /// Filter from date (ISO 8601)
        #[arg(long)]
        from: Option<String>,
        /// Filter to date (ISO 8601)
        #[arg(long)]
        to: Option<String>,
    },

    /// Show single match details
    #[command(name = "match")]
    Match {
        /// Match ID
        #[arg(long)]
        id: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e:#}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let agent = cli.agent.as_deref();
    let _agent_address = config::resolve_agent_address(agent);

    match cli.command {
        Commands::Wallet(args) => commands::wallet::execute(args, agent).await,
        Commands::Register { .. } => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Deposit { .. } => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Balance => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Queue { .. } => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Dequeue { .. } => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Poll => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Submit { .. } => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Status => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Matches { .. } => {
            println!("Not yet implemented");
            Ok(())
        }
        Commands::Match { .. } => {
            println!("Not yet implemented");
            Ok(())
        }
    }
}
