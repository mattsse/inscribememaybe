use alloy_chains::Chain;
use clap::builder::{RangedU64ValueParser, TypedValueParser};
use clap::{Arg, Command, Parser, Subcommand};
use ethers::prelude::Signer;
use ethers::prelude::{Http, LocalWallet, Middleware, Provider, SignerMiddleware};
use ethers::types::{Address, Bytes, TxHash};
use futures::stream::FuturesOrdered;
use futures::{Stream, StreamExt};
use inscribememaybe::{Deploy, InscriptionCalldata, Mint, CALL_DATA_PREFIX};
use serde::de::DeserializeOwned;
use sqlx::migrate::MigrateDatabase;
use std::ffi::OsStr;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tracing::{debug, info, instrument};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    pub sub: Subcommands,
}

/// Subcommands for the CLI
#[derive(Debug, Subcommand)]
pub enum Subcommands {
    /// Mint tokens
    Mint(MintArgs),
    /// Deploy a token
    Deploy {
        /// The mint message
        #[clap(
            value_parser = InscriptionValueParser::<Deploy>::default(),
        )]
        message: Deploy,
        /// The private key to use for signing transactions
        #[clap(long, alias = "pk")]
        private_key: String,
        /// The RPC URL where the transactions will be sent
        #[clap(long)]
        rpc_url: String,
    },
}

/// Mint tokens
#[derive(Debug, Parser)]
pub struct MintArgs {
    /// The mint message, must be a valid JSON string
    #[clap(
        value_parser = InscriptionValueParser::<Mint>::default(),
    )]
    message: Mint,
    /// The private key to use for signing transactions
    #[clap(long, alias = "pk")]
    private_key: String,
    /// The RPC URL where the transactions will be sent
    #[clap(long)]
    rpc_url: String,
    /// The number of transactions to send
    #[clap(
        long,
        value_parser = RangedU64ValueParser::<u64>::new().range(1..),
        default_value_t = 1)
    ]
    transactions: u64,
    /// The number of mints to send concurrently
    #[clap(
        long,
        value_parser = RangedU64ValueParser::<u64>::new().range(1..),
        default_value_t = 16)
    ]
    concurrency: u64,
}

impl MintArgs {
    async fn run(self) -> eyre::Result<()> {
        if self.rpc_url.starts_with("ws") {
            let ws = Provider::connect(&self.rpc_url).await?;
            self.run_mint(ws).await
        } else {
            let provider = Provider::<Http>::try_from(&self.rpc_url)?;
            self.run_mint(provider).await
        }
    }

    async fn run_mint<M>(self, provider: M) -> eyre::Result<()>
    where
        M: Middleware + Send + Sync + Clone + Unpin + 'static,
    {
        let db = Database::connect().await?;

        let chain_id = provider.get_chainid().await?;
        if chain_id.as_u64() == Chain::mainnet().id() {
            println!("it looks like you're targeting ethereum mainnet. To proceed, please acknowledge that you're a degenerate: [y/n]");

            // Read user input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !["y", "yes"].contains(&input.trim().to_lowercase().as_str()) {
                return Ok(());
            }
        }

        let wallet = self
            .private_key
            .parse::<LocalWallet>()?
            .with_chain_id(chain_id.as_u64());

        let address = wallet.address();
        debug!(from=?address, inscription=%self.message, mints=%self.transactions, "start minting");

        let chain = Chain::from(chain_id.as_u64());
        let provider = Arc::new(SignerMiddleware::new(provider, wallet));

        let mut inscriber = Inscriber {
            transactions: Default::default(),
            calldata: self.message.calldata().into(),
            sender: address,
            count: 0,
            max_transactions: self.transactions,
            concurrency: self.concurrency,
            chain_id: chain_id.as_u64(),
            provider,
        };

        while let Some(event) = inscriber.next().await {
            match event {
                InscriptionEvent::Mint {
                    sender,
                    chain_id,
                    tx_hash,
                    calldata,
                } => {
                    if let Some((_, etherscan)) = chain.etherscan_urls() {
                        let mut etherscan_base = etherscan.to_string();
                        if !etherscan_base.ends_with("/") {
                            etherscan_base.push('/');
                        }
                        let tx_url = format!("{}tx/{}", etherscan, tx_hash);
                        info!(%tx_url, "minted");
                    }

                    let _ = db.insert_one(sender, chain_id, tx_hash, calldata).await;
                }
            }
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
#[allow(unused)]
struct InscriptionEntry {
    #[sqlx(json)]
    sender: Address,
    chain_id: u64,
    #[sqlx(json)]
    tx_hash: TxHash,
    #[sqlx(json)]
    calldata: Bytes,
}

/// A simple database for all transactions.
///
/// This way we can actually check what operations have been performed.
#[derive(Clone)]
struct Database(sqlx::SqlitePool);

impl Database {
    /// Connect to an existing database.
    #[instrument]
    pub async fn connect_to(url: &str) -> eyre::Result<Self> {
        if !sqlx::Sqlite::database_exists(url).await.unwrap_or(false) {
            sqlx::Sqlite::create_database(url).await?;
            info!("created database");
        }

        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect(url)
            .await?;

        // run migration
        sqlx::migrate!("./migrations").run(&db).await?;

        info!("connected to database");
        Ok(Self(db))
    }

    /// Connect to an existing database.
    pub async fn connect() -> eyre::Result<Self> {
        Self::connect_to("sqlite://inscribememaybe.sqlite").await
    }

    /// Insert a new inscription.
    #[instrument(skip(self))]
    pub async fn insert_one(
        &self,
        sender: Address,
        chain_id: u64,
        hash: TxHash,
        calldata: Bytes,
    ) -> eyre::Result<()> {
        let res = sqlx::query(
            "INSERT INTO inscriptions (sender, chain_id, tx_hash, calldata) VALUES ($1, $2, $3, $4)")
            .bind(format!("{:?}", sender))
            .bind(chain_id as i64)
            .bind(format!("{:?}", hash))
            .bind(format!("{:?}", calldata))
            .execute(&self.0).await?;
        info!(?res, "inserted inscription");

        Ok(())
    }
}

/// A value parser for deserializing JSON values.
///
/// if the value starts with [CALL_DATA_PREFIX] it will be stripped before deserialization.
#[derive(Debug, Clone)]
struct InscriptionValueParser<T>(PhantomData<T>);

impl<T> Default for InscriptionValueParser<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> TypedValueParser for InscriptionValueParser<T>
where
    T: DeserializeOwned + Send + Sync + Clone + 'static,
{
    type Value = T;

    fn parse_ref(
        &self,
        _cmd: &Command,
        _arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let val = value
            .to_str()
            .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidUtf8))?;
        let raw = val.trim_start_matches(CALL_DATA_PREFIX);

        serde_json::from_str(raw)
            .map_err(|err| clap::Error::raw(clap::error::ErrorKind::InvalidValue, err))
    }
}

/// Handles inscriptions.
struct Inscriber<M> {
    /// in progress transactions
    transactions: FuturesOrdered<Pin<Box<dyn Future<Output = eyre::Result<TxHash>>>>>,
    /// The call data to send
    calldata: Bytes,
    sender: Address,
    /// how many transactions we sent already
    count: u64,
    /// How many transactions to send
    max_transactions: u64,
    /// How many transactions to send concurrently
    concurrency: u64,
    /// The targeted chain id
    chain_id: u64,
    /// The provider to use
    provider: M,
}

impl<M> Stream for Inscriber<M>
where
    M: Middleware + Send + Sync + Clone + Unpin + 'static,
{
    type Item = InscriptionEvent;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
enum InscriptionEvent {
    Mint {
        sender: Address,
        chain_id: u64,
        tx_hash: TxHash,
        calldata: Bytes,
    },
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("inscribememaybe=info".parse()?),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    match args.sub {
        Subcommands::Mint(args) => {
            args.run().await?;
        }
        Subcommands::Deploy { .. } => {}
    }

    // let _wallet = args.private_key.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mint() {
        let _args = Args::parse_from([
            "inscribememaybe",
            "mint",
            r#"{"p":"fair-20","op":"mint","tick":"brr","amt":"1000"}"#,
            "--pk",
            "0xdeadbeef",
            "--rpc-url",
            "http://localhost:9933",
        ]);
    }

    #[tokio::test]
    #[ignore]
    async fn test_insert_one() {
        let db = Database::connect().await.unwrap();
        db.insert_one(
            Default::default(),
            1,
            Default::default(),
            Default::default(),
        )
        .await
        .unwrap();
    }
}
