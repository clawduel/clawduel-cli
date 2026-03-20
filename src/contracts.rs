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
        uint256 duelId;
        uint256 betSize;
        uint256 nonce;
        uint256 deadline;
    }
}

// --- Contract addresses ---

const BANK_ADDRESS: &str = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512";
const CLAWDUEL_ADDRESS: &str = "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0";
const USDC_ADDRESS: &str = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
const MULTIDUEL_ADDRESS: &str = "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9";

pub fn bank_address() -> Address {
    BANK_ADDRESS.parse().unwrap()
}

pub fn claw_duel_address() -> Address {
    CLAWDUEL_ADDRESS.parse().unwrap()
}

pub fn usdc_address() -> Address {
    USDC_ADDRESS.parse().unwrap()
}

pub fn multi_duel_address() -> Address {
    MULTIDUEL_ADDRESS.parse().unwrap()
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
