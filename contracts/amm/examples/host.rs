use amm::*;
use token::*;
use clap::Parser;
use sdk::{Blob, BlobData, BlobTransaction, Identity, ContractName};
use std::time::Instant;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Arguments for CLI tool
#[derive(Parser, Debug)]
struct Args {
    /// Create a new liquidity pool
    #[arg(long)]
    create_pool: bool,

    /// Token A contract name
    #[arg(long)]
    token_a: Option<String>,

    /// Token B contract name
    #[arg(long)]
    token_b: Option<String>,

    /// Amount of token A
    #[arg(long, default_value_t = 1000)]
    amount_a: u128,

    /// Amount of token B
    #[arg(long, default_value_t = 1000)]
    amount_b: u128,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(true))
        .init();

    let args = Args::parse();

    // Create a new AMM instance
    let mut amm = Amm::default();

    if args.create_pool {
        if let (Some(token_a_str), Some(token_b_str)) = (args.token_a.as_ref(), args.token_b.as_ref()) {
            let token_a = ContractName(token_a_str.clone());
            let token_b = ContractName(token_b_str.clone());
            let amount_a = args.amount_a;
            let amount_b = args.amount_b;

            // Create a pool action
            let action = AmmAction::CreatePool {
                token_a: token_a.clone(),
                token_b: token_b.clone(),
                amount_a,
                amount_b,
            };

            // Simulate the execution
            let start = Instant::now();
            let result = amm.create_pool(token_a, token_b, amount_a, amount_b)?;
            let elapsed = start.elapsed();

            println!("Result: {}", result);
            println!("Execution time: {:?}", elapsed);
            println!("New state: {:?}", amm);
        } else {
            println!("Please specify token_a and token_b to create a pool");
        }
    } else {
        println!("Please use --create-pool to create a new liquidity pool");
    }

    Ok(())
} 