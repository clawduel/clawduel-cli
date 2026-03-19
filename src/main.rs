use std::process::ExitCode;

use clap::{Parser, Subcommand};
use clawduel_cli::{commands, config, http::HttpClient, wallet};

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

    // Wallet command doesn't need a loaded wallet
    if let Commands::Wallet(args) = cli.command {
        return commands::wallet::execute(args, agent).await;
    }

    // All other commands require a wallet
    let keystores_dir = wallet::default_keystores_dir()?;
    let agent_address = config::resolve_agent_address(agent);
    let signer = wallet::load_wallet(agent_address.as_deref(), None, &keystores_dir, None)?;
    let address = signer.address();
    let private_key_hex = hex::encode(signer.to_bytes());

    let backend_url = config::resolve_backend_url(None);
    let rpc_url = config::resolve_rpc_url(None);

    match cli.command {
        Commands::Wallet(_) => unreachable!(),

        Commands::Register { nickname } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::register::execute(&client, &nickname).await
        }

        Commands::Deposit { amount } => {
            commands::deposit::execute(amount, &address, &signer, &rpc_url).await
        }

        Commands::Balance => {
            commands::balance::execute(&address, &rpc_url).await
        }

        Commands::Queue { bet_tier, timeout } => {
            let client = HttpClient::new(&backend_url, signer.clone(), address, &private_key_hex)?;
            commands::queue::execute(&client, bet_tier, timeout, &address, &signer, &rpc_url).await
        }

        Commands::Dequeue { bet_tier } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::dequeue::execute(&client, bet_tier).await
        }

        Commands::Poll => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::poll::execute(&client, &address).await
        }

        Commands::Submit {
            match_id,
            prediction,
        } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::submit::execute(&client, &match_id, &prediction).await
        }

        Commands::Status => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::status::execute(&client, &address, &rpc_url).await
        }

        Commands::Matches {
            status,
            page,
            category,
            from,
            to,
        } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            let filters = commands::matches::MatchFilters {
                status,
                page,
                category,
                from,
                to,
            };
            commands::matches::execute(&client, filters).await
        }

        Commands::Match { id } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::match_detail::execute(&client, &id).await
        }
    }
}
