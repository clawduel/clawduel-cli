//! Contract addresses, ABIs, and provider construction for on-chain interactions.

use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use anyhow::{Context, Result};

// --- Contract ABIs ---

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
    }
}

sol! {
    #[sol(rpc)]
    interface IBank {
        function deposit(uint256 amount) external;
        function balanceOf(address account) external view returns (uint256);
        function lockedBalanceOf(address account) external view returns (uint256);
    }
}

sol! {
    #[sol(rpc)]
    interface IClawDuel {
        function usedNonces(address agent, uint256 nonce) external view returns (bool);
    }
}

sol! {
    #[sol(rpc)]
    interface IMultiDuel {
        function usedNonces(address agent, uint256 nonce) external view returns (bool);
    }
}

// --- EIP-712 types ---

sol! {
    #[derive(Debug)]
    struct JoinDuelAttestation {
        address agent;
        uint256 betTier;
        uint256 nonce;
        uint256 deadline;
    }
}

sol! {
    #[derive(Debug)]
    struct JoinMultiAttestation {
        address agent;
        uint256 betTier;
        uint256 nonce;
        uint256 deadline;
    }
}

// --- Default contract addresses ---

const DEFAULT_BANK_ADDRESS: &str = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";
const DEFAULT_CLAWDUEL_ADDRESS: &str = "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0";
const DEFAULT_USDC_ADDRESS: &str = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
const DEFAULT_MULTIDUEL_ADDRESS: &str = "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9";

/// Resolved contract addresses.
pub struct ContractAddresses {
    pub bank: Address,
    pub claw_duel: Address,
    pub usdc: Address,
    pub multi_duel: Address,
}

/// Resolve contract addresses from env vars with defaults.
pub fn resolve_addresses() -> Result<ContractAddresses> {
    let bank = resolve_address("CLAW_BANK_ADDRESS", DEFAULT_BANK_ADDRESS)?;
    let claw_duel = resolve_address("CLAW_CLAWDUEL_ADDRESS", DEFAULT_CLAWDUEL_ADDRESS)?;
    let usdc = resolve_address("CLAW_USDC_ADDRESS", DEFAULT_USDC_ADDRESS)?;
    let multi_duel = resolve_address("CLAW_MULTIDUEL_ADDRESS", DEFAULT_MULTIDUEL_ADDRESS)?;

    Ok(ContractAddresses {
        bank,
        claw_duel,
        usdc,
        multi_duel,
    })
}

fn resolve_address(env_var: &str, default: &str) -> Result<Address> {
    let raw = std::env::var(env_var).unwrap_or_else(|_| default.to_string());
    raw.parse::<Address>()
        .context(format!("Invalid address for {env_var}: {raw}"))
}

/// Create an alloy HTTP provider from an RPC URL.
pub async fn create_provider(
    rpc_url: &str,
) -> Result<impl Provider + Clone> {
    let url = rpc_url
        .parse()
        .context(format!("Invalid RPC URL: {rpc_url}"))?;
    let provider = ProviderBuilder::new().connect_http(url);
    Ok(provider)
}

/// Parse a USDC amount (f64 in USDC units) to U256 with 6 decimals.
pub fn parse_usdc(amount: f64) -> U256 {
    let micro = (amount * 1_000_000.0).round() as u64;
    U256::from(micro)
}

/// Format a U256 USDC amount (6 decimals) to a human-readable string.
pub fn format_usdc(amount: U256) -> String {
    let micro: u64 = amount.try_into().unwrap_or(0);
    let whole = micro / 1_000_000;
    let frac = micro % 1_000_000;
    format!("{whole}.{frac:06}")
}
