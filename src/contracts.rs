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
        function withdraw(uint256 amount) external;
        function balanceOf(address account) external view returns (uint256);
        function withdrawalNonces(address account) external view returns (uint256);
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

sol! {
    #[derive(Debug)]
    struct ReceiveWithAuthorization {
        address from;
        address to;
        uint256 value;
        uint256 validAfter;
        uint256 validBefore;
        bytes32 nonce;
    }
}

sol! {
    #[derive(Debug)]
    struct DepositAuthorization {
        address agent;
        uint256 creditAmount;
        uint256 feeAmount;
        bytes32 authorizationNonce;
        uint256 deadline;
    }
}

sol! {
    #[derive(Debug)]
    struct WithdrawAuthorization {
        address agent;
        address recipient;
        uint256 amount;
        uint256 feeAmount;
        uint256 nonce;
        uint256 deadline;
    }
}

// --- Contract addresses ---

const PRIZE_POOL_ADDRESS: &str = "0x98C76d1ef4c2597E8BdadC5C08D4a5AA8Ae25dD6";
const COMPETITION_ADDRESS: &str = "0x970f8f62E6b2bdDb74B190B9c5e2f9dC64080544";
const USDC_ADDRESS: &str = "0xCc535B7A307e662363332cf46C8e49be6b878c53";
const MULTI_COMPETITION_ADDRESS: &str = "0x545e26ba2413C8975183ff7Eb143E7076369A16f";

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
