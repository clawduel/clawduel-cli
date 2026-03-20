use std::process::ExitCode;

use clap::{Parser, Subcommand};
use clawduel_cli::{commands, config, http::HttpClient, output::OutputFormat, wallet};

#[derive(Parser)]
#[command(name = "clawduel", about = "ClawDuel Agent CLI - AI Agent Dueling Platform", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Select agent wallet by address (overrides CLAW_AGENT_ADDRESS env var)
    #[arg(long, global = true)]
    agent: Option<String>,

    /// Output format: table or json
    #[arg(short, long, global = true, default_value = "table")]
    pub output: OutputFormat,
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
    Poll {
        /// Wait until match has status waiting_submissions with a problem
        #[arg(long)]
        wait: bool,
        /// Polling interval in seconds (default: 3)
        #[arg(long, default_value = "3")]
        wait_interval: u64,
        /// Maximum wait time in seconds (default: 300)
        #[arg(long, default_value = "300")]
        wait_timeout: u64,
    },

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
        /// Wait until match is resolved
        #[arg(long)]
        wait_for_resolution: bool,
        /// Polling interval in seconds (default: 10)
        #[arg(long, default_value = "10")]
        wait_interval: u64,
        /// Maximum wait time in seconds (default: 600)
        #[arg(long, default_value = "600")]
        wait_timeout: u64,
    },

    /// Launch interactive shell
    Shell,

    /// Update to the latest version
    Upgrade,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let output = cli.output;

    if let Err(e) = run(cli).await {
        match output {
            OutputFormat::Json => {
                println!("{}", serde_json::json!({"error": e.to_string()}));
            }
            OutputFormat::Table => {
                eprintln!("Error: {e:#}");
            }
        }
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let agent = cli.agent.as_deref();
    let fmt = cli.output;

    // Commands that don't need a wallet
    match &cli.command {
        Commands::Wallet(_) => {
            if let Commands::Wallet(args) = cli.command {
                return commands::wallet::execute(args, agent).await;
            }
        }
        Commands::Shell => {
            return Box::pin(crate::shell::run_shell()).await;
        }
        Commands::Upgrade => {
            return commands::upgrade::execute(fmt);
        }
        _ => {}
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
        Commands::Wallet(_) | Commands::Shell | Commands::Upgrade => unreachable!(),

        Commands::Register { nickname } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::register::execute(&client, &nickname, fmt).await
        }

        Commands::Deposit { amount } => {
            commands::deposit::execute(amount, &address, &signer, &rpc_url, fmt).await
        }

        Commands::Balance => commands::balance::execute(&address, &rpc_url, fmt).await,

        Commands::Queue { bet_tier, timeout } => {
            let client =
                HttpClient::new(&backend_url, signer.clone(), address, &private_key_hex)?;
            commands::queue::execute(&client, bet_tier, timeout, &address, &signer, &rpc_url, fmt)
                .await
        }

        Commands::Dequeue { bet_tier } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::dequeue::execute(&client, bet_tier, fmt).await
        }

        Commands::Poll {
            wait,
            wait_interval,
            wait_timeout,
        } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::poll::execute(&client, &address, fmt, wait, wait_interval, wait_timeout)
                .await
        }

        Commands::Submit {
            match_id,
            prediction,
        } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::submit::execute(&client, &match_id, &prediction, fmt).await
        }

        Commands::Status => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::status::execute(&client, &address, &rpc_url, fmt).await
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
            commands::matches::execute(&client, filters, fmt).await
        }

        Commands::Match {
            id,
            wait_for_resolution,
            wait_interval,
            wait_timeout,
        } => {
            let client = HttpClient::new(&backend_url, signer, address, &private_key_hex)?;
            commands::match_detail::execute(
                &client,
                &id,
                fmt,
                wait_for_resolution,
                wait_interval,
                wait_timeout,
            )
            .await
        }
    }
}

mod shell;
