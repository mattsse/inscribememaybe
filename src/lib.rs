//! Ethereum inscriptions

#![doc(issue_tracker_base_url = "https://github.com/mattsse/inscribememaybe/issues/")]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use ethers::types::Address;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DisplayFromStr};
use std::{fmt, str::FromStr, string::FromUtf8Error};

pub use protocol::*;

mod protocol;

/// The prefix for json calldata
pub const CALL_DATA_PREFIX: &str = "data:,";

/// A helper trait for encoding inscription calldata
pub trait InscriptionCalldata {
    /// Returns the calldata for the inscription
    ///
    /// Note: This must be valid utf-8 and must contain the prefix [CALL_DATA_PREFIX]
    fn calldata(&self) -> Vec<u8>;

    /// Returns the calldata as a UTF8 string
    ///
    /// # Panics
    ///
    /// If the calldata is not valid utf-8
    fn calldata_string(&self) -> String
    where
        Self: Sized,
    {
        self.try_calldata_string().expect("Valid utf-8")
    }

    /// Returns the calldata as a UTF8 string
    fn try_calldata_string(&self) -> Result<String, FromUtf8Error>
    where
        Self: Sized,
    {
        String::from_utf8(self.calldata())
    }
}

macro_rules! impl_inscription_calldata {
    ($($t:ty),*) => {
        $(
            impl InscriptionCalldata for $t {
                fn calldata(&self) -> Vec<u8> {
                    let mut buf = CALL_DATA_PREFIX.as_bytes().to_vec();
                    serde_json::to_writer(&mut buf, self).expect("Valid json");
                    buf
                }
            }

            impl fmt::Display for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.calldata_string().fmt(f)
                }
            }

        )*
    };
}

impl_inscription_calldata!(Deploy, Mint, Transfer);

/// Represents a deploy operation for inscribing data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deploy {
    /// Represents the protocol, and here the ERC-20 protocol is used.
    pub p: Protocol,
    /// Represents the current token name to be deployed.
    pub tick: String,
    /// Represents the maximum issuance.
    pub max: u64,
    /// Represents the maximum amount that can be mined per mining operation.
    pub lim: u64,
}

impl Serialize for Deploy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut x = serializer.serialize_struct("Deploy", 5)?;
        x.serialize_field("p", &self.p)?;
        x.serialize_field("op", &"deploy")?;
        x.serialize_field("tick", &self.tick)?;
        x.serialize_field("max", &self.max.to_string())?;
        x.serialize_field("lim", &self.lim.to_string())?;
        x.end()
    }
}

impl<'de> Deserialize<'de> for Deploy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[serde_as]
        #[derive(Deserialize)]
        struct DeployOp {
            p: Protocol,
            op: Op,
            tick: String,
            #[serde_as(as = "DisplayFromStr")]
            max: u64,
            #[serde_as(as = "DisplayFromStr")]
            lim: u64,
        }

        let deploy = DeployOp::deserialize(deserializer)?;
        if !deploy.op.is_deploy() {
            return Err(serde::de::Error::custom(format!(
                "Invalid operation: {}, expected deploy",
                deploy.op
            )));
        }
        Ok(Deploy { p: deploy.p, tick: deploy.tick, max: deploy.max, lim: deploy.lim })
    }
}

/// Represents a mint operation for inscribing data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mint {
    /// Represents the protocol, and here the ERC-20 protocol is used.
    pub p: Protocol,
    /// Represents the current token name to be deployed.
    pub tick: String,
    /// The _unique_ id to use
    pub id: Option<String>,
    /// Represents the maximum amount that can be mined per mining operation.
    pub amt: u64,
}

impl Serialize for Mint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut x = serializer.serialize_struct("Mint", 4 + self.id.is_some() as usize)?;
        x.serialize_field("p", &self.p)?;
        x.serialize_field("op", &"mint")?;
        x.serialize_field("tick", &self.tick)?;
        if let Some(id) = &self.id {
            x.serialize_field("id", id)?;
        }
        x.serialize_field("amt", &self.amt.to_string())?;
        x.end()
    }
}

impl<'de> Deserialize<'de> for Mint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[serde_as]
        #[derive(Deserialize)]
        struct MintOp {
            p: Protocol,
            op: Op,
            tick: String,
            id: Option<String>,
            #[serde_as(as = "DisplayFromStr")]
            amt: u64,
        }

        let mint = MintOp::deserialize(deserializer)?;
        if !mint.op.is_mint() {
            return Err(serde::de::Error::custom(format!(
                "Invalid operation: {}, expected mint",
                mint.op
            )));
        }
        Ok(Mint { p: mint.p, tick: mint.tick, id: mint.id, amt: mint.amt })
    }
}

/// Represents a mint operation for inscribing data.
// TODO: how may transfer variants are there?
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    /// Represents the protocol, and here the ERC-20 protocol is used.
    pub p: Protocol,
    /// Represents the current token name to be deployed.
    pub tick: String,
    /// Target of the transfer
    pub to: Vec<TransferItem>,
}

impl Serialize for Transfer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut x = serializer.serialize_struct("Transfer", 3)?;
        x.serialize_field("p", &self.p)?;
        x.serialize_field("op", &"transfer")?;
        x.serialize_field("tick", &self.tick)?;
        x.serialize_field("to", &self.to)?;
        x.end()
    }
}

impl<'de> Deserialize<'de> for Transfer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TransferOp {
            p: Protocol,
            op: Op,
            tick: String,
            to: Vec<TransferItem>,
        }

        let transfer = TransferOp::deserialize(deserializer)?;
        if !transfer.op.is_transfer() {
            return Err(serde::de::Error::custom(format!(
                "Invalid operation: {}, expected transfer",
                transfer.op
            )));
        }
        Ok(Transfer { p: transfer.p, tick: transfer.tick, to: transfer.to })
    }
}

/// How much to transfer to whom
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferItem {
    /// recipient of the transfer
    pub recv: Address,
    /// amount to transfer
    // #[serde_as(as = "DisplayFromStr")] // TODO why is this apparently a number?
    pub amt: i64,
}

/// Represents operations for inscribing data.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// Deploy a new token.
    Deploy,
    /// Mint a new token.
    Mint,
    /// Transfer a token
    Transfer,
}

impl Op {
    /// Returns true if the operation is `Deploy`.
    pub const fn is_deploy(&self) -> bool {
        matches!(self, Op::Deploy)
    }

    /// Returns true if the operation is `Mint`.
    pub const fn is_mint(&self) -> bool {
        matches!(self, Op::Mint)
    }

    /// Returns true if the operation is `Transfer`.
    pub const fn is_transfer(&self) -> bool {
        matches!(self, Op::Transfer)
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Deploy => write!(f, "deploy"),
            Op::Mint => write!(f, "mint"),
            Op::Transfer => {
                write!(f, "transfer")
            }
        }
    }
}

impl FromStr for Op {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deploy" => Ok(Op::Deploy),
            "mint" => Ok(Op::Mint),
            "transfer" => Ok(Op::Transfer),
            s => Err(format!("invalid operation: {s}")),
        }
    }
}

impl Serialize for Op {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Op {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let op = String::deserialize(deserializer)?;
        Op::from_str(&op).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calldata() {
        let json_data =
            r#"{"p":"erc-20","op":"deploy","tick":"gwei","max":"21000000","lim":"1000"}"#;
        let operation: Deploy =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");
        let calldata = operation.calldata_string();
        assert_eq!(
            calldata,
            r#"data:,{"p":"erc-20","op":"deploy","tick":"gwei","max":"21000000","lim":"1000"}"#
        );
    }

    #[test]
    fn test_transfer_serde() {
        let json_data = r#"{"p":"osc-20","op":"transfer","tick":"osct","to":[{"recv":"0x8D4E4Ee435a2FE82A037ba10d4486049bADbCdB2","amt":-1000}]}"#;
        let transfer: Transfer = serde_json::from_str(json_data).unwrap();
        assert_eq!(
            transfer,
            Transfer {
                p: NamedProtocol::Osc_20.into(),
                tick: "osct".to_string(),
                to: vec![TransferItem {
                    recv: "0x8D4E4Ee435a2FE82A037ba10d4486049bADbCdB2".parse().unwrap(),
                    amt: -1000,
                }],
            }
        );
    }

    #[test]
    fn test_deploy_serde() {
        let json_data =
            r#"{"p":"erc-20","op":"deploy","tick":"gwei","max":"21000000","lim":"1000"}"#;

        let operation: Deploy =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let serialized_json =
            serde_json::to_string(&operation).expect("Failed to serialize to JSON");

        let expected_operation =
            Deploy { p: "erc-20".into(), tick: "gwei".to_string(), max: 21000000, lim: 1000 };

        assert_eq!(operation, expected_operation);

        let expected_json =
            r#"{"p":"erc-20","op":"deploy","tick":"gwei","max":"21000000","lim":"1000"}"#;
        assert_eq!(serialized_json, expected_json);
    }

    #[test]
    fn test_display_from_str() {
        // Test Display
        assert_eq!(Op::Deploy.to_string(), "deploy");
        assert_eq!(Op::Mint.to_string(), "mint");

        // Test FromStr
        assert_eq!(Op::from_str("deploy"), Ok(Op::Deploy));
        assert_eq!(Op::from_str("mint"), Ok(Op::Mint));
        assert!(Op::from_str("invalid").is_err());
    }
}
