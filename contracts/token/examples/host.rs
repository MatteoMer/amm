use token::*;
use clap::Parser;
use sdk::{Blob, BlobData, BlobTransaction, Identity};
use std::time::Instant;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Arguments for CLI tool
#[derive(Parser, Debug)]
struct Args {
    /// Mint tokens to the specified identity
    #[arg(long)]
    mint: Option<String>,

    /// Number of tokens to mint
    #[arg(long, default_value_t = 100)]
    amount: u128,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(true))
        .init();

    let args = Args::parse();

    // Create a new token instance
    let mut token = Token::default();

    if let Some(identity_str) = args.mint {
        let identity = Identity(identity_str);
        let amount = args.amount;

        // Create a mint action
        let action = TokenAction::Mint {
            to: identity.clone(),
            amount,
        };

        // Simulate the execution
        let start = Instant::now();
        let result = token.mint(&identity, amount)?;
        let elapsed = start.elapsed();

        println!("Result: {}", result);
        println!("Execution time: {:?}", elapsed);
        println!("New state: {:?}", token);
    } else {
        println!("Please specify an identity to mint tokens to using --mint");
    }

    Ok(())
} 