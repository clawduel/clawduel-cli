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
    interface IPrizePool {
        function deposit(uint256 amount) external;
        function balanceOf(address account) external view returns (uint256);
    }
}

sol! {
    #[sol(rpc)]
    interface ICompetition {
        function usedNonces(address agent, uint256 nonce) external view returns (bool);
    }
}

sol! {
    #[sol(rpc)]
    interface IMultiCompetition {
        function usedNonces(address agent, uint256 nonce) external view returns (bool);
    }
}

// --- EIP-712 types ---

sol! {
    #[derive(Debug)]
    struct JoinCompetitionAttestation {
        address agent;
        uint256 entryFee;
        uint256 nonce;
        uint256 deadline;
    }
}

sol! {
    #[derive(Debug)]
    struct JoinMultiCompetitionAttestation {
        address agent;
        uint256 competitionId;
        uint256 entryFee;
        uint256 nonce;
        uint256 deadline;
    }
}

// --- Contract addresses ---

const PRIZE_POOL_ADDRESS: &str = "0xe8a75775cd5d7Cab1A83f086115Ec3D4ad935Fac";
const COMPETITION_ADDRESS: &str = "0x2C480f7cE1E4434B65Aee606f4Bb5A8287D63716";
const USDC_ADDRESS: &str = "0x1dF107364ac1c0D992Ae763aE1CCccFA197Ee304";
const MULTI_COMPETITION_ADDRESS: &str = "0xEaAd5F9912612261CA1937FF7A8Edab83A401bb4";

pub fn prize_pool_address() -> Address {
    PRIZE_POOL_ADDRESS.parse().unwrap()
}

pub fn competition_address() -> Address {
    COMPETITION_ADDRESS.parse().unwrap()
}

pub fn usdc_address() -> Address {
    USDC_ADDRESS.parse().unwrap()
}

pub fn multi_competition_address() -> Address {
    MULTI_COMPETITION_ADDRESS.parse().unwrap()
}

/// Create an alloy HTTP provider from an RPC URL.
pub async fn create_provider(rpc_url: &str) -> Result<impl Provider + Clone> {
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
