use clap::{Parser, Subcommand};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    pub sub: Subcommands,
}

/// Sucommands for the CLI
#[derive(Debug, Subcommand)]
pub enum Subcommands {
    /// Mint tokens
    Mint {
        /// The private key to use for signing transactions
        #[clap(long, alias = "pk")]
        private_key: String,
        /// The mint message
        #[clap(long)]
        message: String,
        /// The RPC URL where the transactions will be sent
        #[clap(long)]
        rpc_url: String,
        /// The number of transactions to send
        #[clap(long)]
        transactions: u64,
    },
    /// Deploy a token
    Deploy {
        /// The private key to use for signing transactions
        #[clap(long, alias = "pk")]
        private_key: String,
        /// The mint message
        #[clap(long)]
        message: String,
        /// The RPC URL where the transactions will be sent
        #[clap(long)]
        rpc_url: String,
    },
}

/// A simple database for all transactions.
///
/// This way we can actually check what operations have been performed.
struct Database(sqlx::SqlitePool);

impl Database {
    /// Create a new database.
    pub async fn new() -> eyre::Result<Self> {
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite://inscribememaybe.sqlite")
            .await?;
        Ok(Self(db))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let args = Args::parse();

    let _db = Database::new().await?;
    // let provider = Provider::<Http>::try_from(args.rpc_url)?;
    // let chain_id = provider.get_chainid().await?;

    // let _wallet = args.private_key.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());

    Ok(())
}
