use alloy_chains::Chain;
use clap::{
    builder::{RangedU64ValueParser, TypedValueParser},
    Arg, Command, Parser, Subcommand,
};
use ethers::{
    prelude::{
        transaction::eip2718::TypedTransaction, Http, LocalWallet, Middleware, Provider, Signer,
        SignerMiddleware, TransactionReceipt,
    },
    types::{Address, Bytes, TransactionRequest, TxHash},
};
use futures::{stream::FuturesUnordered, Stream, StreamExt};
use inscribememaybe::{Deploy, InscriptionCalldata, Mint, CALL_DATA_PREFIX};
use serde::de::DeserializeOwned;
use sqlx::migrate::MigrateDatabase;
use std::{
    ffi::OsStr,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};
use tracing::{debug, info, instrument, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        if Chain::mainnet() == chain_id.as_u64() {
            println!("it looks like you're targeting ethereum mainnet. to proceed, acknowledge that you're a degenerate and willingly continue at your own risk.: [y/n]");

            // Read user input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !["y", "yes"].contains(&input.trim().to_lowercase().as_str()) {
                return Ok(());
            }
        }

        if self.transactions > 1 && self.message.id.is_some() {
            println!("you're about to mint tokens with the same inscription id. this is probably not what you want. continue anyway?: [y/n]");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !["y", "yes"].contains(&input.trim().to_lowercase().as_str()) {
                return Ok(());
            }
        }

        let wallet = self.private_key.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());

        let address = wallet.address();
        let nonce = provider.get_transaction_count(wallet.address(), None).await?;

        debug!(from=?address, nonce=%nonce.as_u64(), inscription=%self.message, mints=%self.transactions, "start minting");

        let chain = Chain::from(chain_id.as_u64());
        let provider = Arc::new(SignerMiddleware::new(provider, wallet));

        let mut inscriber = Inscriber {
            pending: Default::default(),
            calldata: self.message.calldata().into(),
            sender: address,
            count: 0,
            highest_nonce: nonce.as_u64(),
            max_transactions: self.transactions,
            concurrency: self.concurrency as usize,
            chain_id: chain_id.as_u64(),
            provider,
        };

        let mut mints = 0;
        while let Some(event) = inscriber.next().await {
            match event {
                InscriptionEvent::Mint { sender, chain_id, calldata, receipt } => {
                    let tx_hash = receipt.transaction_hash;
                    let block = receipt.block_number.unwrap_or_default().as_u64();
                    if let Some((_, etherscan)) = chain.etherscan_urls() {
                        let tx_url = format!("{}/tx/{:?}", etherscan, tx_hash);
                        info!(%tx_url, %block, "minted");
                    } else {
                        info!(hash=?tx_hash, %block, "minted");
                    }

                    let _ = db.insert_one(sender, chain_id, tx_hash, calldata).await;

                    mints += 1;
                }
            }
        }

        info!(%mints, "finished minting");

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

        let db = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect(url).await?;

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
        debug!(?res, "inserted inscription");

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
        let val =
            value.to_str().ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidUtf8))?;
        let raw = val.trim_start_matches(CALL_DATA_PREFIX);

        serde_json::from_str(raw)
            .map_err(|err| clap::Error::raw(clap::error::ErrorKind::InvalidValue, err))
    }
}

/// Handles inscriptions.
///
/// TODO
struct Inscriber<M> {
    /// in progress transactions
    // TODO timestamp these and rebroadcast if they take too long
    pending: FuturesUnordered<Pin<Box<dyn Future<Output = InscriptionResult>>>>,
    /// The call data to send
    calldata: Bytes,
    sender: Address,
    /// how many transactions we sent already
    count: u64,
    /// The next nonce to use
    highest_nonce: u64,
    /// How many transactions to send
    max_transactions: u64,
    /// How many transactions to send concurrently
    concurrency: usize,
    /// The targeted chain id
    chain_id: u64,
    /// The provider to use
    ///
    /// Caution: we expect this to sign the transaction
    provider: M,
}

impl<M> Inscriber<M> {
    /// Returns the next transaction to send.
    fn next_transaction(&mut self) -> TypedTransaction {
        TransactionRequest::new()
            .to(self.sender)
            .value(0u64)
            // This should be sufficient
            .gas(22200)
            .nonce(self.highest_nonce)
            .data(self.calldata.clone())
            .into()
    }
}

impl<M> Inscriber<M>
where
    M: Middleware + Send + Sync + Clone + Unpin + 'static,
{
    /// This starts sending the given transaction
    fn start_transaction(&mut self, tx: TypedTransaction) {
        let provider = self.provider.clone();
        let nonce = tx.nonce().expect("nonce is set").as_u64();
        let fut = async move {
            let pending = provider.send_transaction(tx.clone(), None).await;
            let res = match pending {
                Ok(pending) => pending.await.map_err(Into::into),
                Err(err) => Err(err.into()),
            };
            InscriptionResult { tx, nonce, res }
        };
        self.pending.push(Box::pin(fut));
    }
}

impl<M> Stream for Inscriber<M>
where
    M: Middleware + Send + Sync + Clone + Unpin + 'static,
{
    type Item = InscriptionEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if this.count >= this.max_transactions && this.pending.is_empty() {
                // we're done
                return Poll::Ready(None);
            }

            while this.pending.len() < this.concurrency && this.count < this.max_transactions {
                let tx = this.next_transaction();
                this.start_transaction(tx);
                this.highest_nonce += 1;
                this.count += 1;
            }

            if let Some(res) = ready!(this.pending.poll_next_unpin(cx)) {
                let InscriptionResult { tx, res, nonce } = res;
                match res {
                    Ok(Some(receipt)) => {
                        debug!(?receipt, "minted");
                        return Poll::Ready(Some(InscriptionEvent::Mint {
                            receipt,
                            sender: this.sender,
                            chain_id: this.chain_id,
                            calldata: this.calldata.clone(),
                        }));
                    }
                    Ok(None) => {
                        warn!(%nonce, "failed to get tx receipt; resending");
                        this.start_transaction(tx)
                    }
                    Err(err) => {
                        // TODO better error handling here
                        debug!(%err, %nonce, "failed to mint; resending");
                        this.start_transaction(tx)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct InscriptionResult {
    tx: TypedTransaction,
    nonce: u64,
    res: eyre::Result<Option<TransactionReceipt>>,
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
enum InscriptionEvent {
    Mint { receipt: TransactionReceipt, sender: Address, chain_id: u64, calldata: Bytes },
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
        Subcommands::Deploy { .. } => {
            eprintln!("deploying tokens is not yet supported");
        }
    }

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
        db.insert_one(Default::default(), 1, Default::default(), Default::default()).await.unwrap();
    }
}
