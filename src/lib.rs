//! Ethereum inscriptions

#![doc(issue_tracker_base_url = "https://github.com/mattsse/inscribememaybe/issues/")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;
use std::fmt;
use std::str::FromStr;

/// Represents a deploy operation for inscribing data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deploy {
    /// Represents the protocol, and here the ERC-20 protocol is used.
    pub p: String,
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
        use serde_with::{serde_as, DisplayFromStr};

        #[serde_as]
        #[derive(Deserialize)]
        struct DeployOp {
            p: String,
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
        Ok(Deploy {
            p: deploy.p,
            tick: deploy.tick,
            max: deploy.max,
            lim: deploy.lim,
        })
    }
}

/// Represents a mint operation for inscribing data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mint {
    /// Represents the protocol, and here the ERC-20 protocol is used.
    pub p: String,
    /// Represents the current token name to be deployed.
    pub tick: String,
    /// The _unique_ id to use
    pub id: String,
    /// Represents the maximum amount that can be mined per mining operation.
    pub amt: u64,
}

impl Serialize for Mint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut x = serializer.serialize_struct("Deploy", 5)?;
        x.serialize_field("p", &self.p)?;
        x.serialize_field("op", &"mint")?;
        x.serialize_field("tick", &self.tick)?;
        x.serialize_field("id", &self.id)?;
        x.serialize_field("amt", &self.amt.to_string())?;
        x.end()
    }
}

impl<'de> Deserialize<'de> for Mint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde_with::{serde_as, DisplayFromStr};

        #[serde_as]
        #[derive(Deserialize)]
        struct MintOp {
            p: String,
            op: Op,
            tick: String,
            id: String,
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
        Ok(Mint {
            p: mint.p,
            tick: mint.tick,
            id: mint.id,
            amt: mint.amt,
        })
    }
}

/// Represents operations for inscribing data.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// Deploy a new token.
    Deploy,
    /// Mint a new token.
    Mint,
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
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Deploy => write!(f, "deploy"),
            Op::Mint => write!(f, "mint"),
        }
    }
}

impl FromStr for Op {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deploy" => Ok(Op::Deploy),
            "mint" => Ok(Op::Mint),
            _ => Err("Invalid operation"),
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

    // Test function for serialization and deserialization.
    #[test]
    fn test_serialization_deserialization() {
        // Example JSON
        let json_data =
            r#"{"p":"erc-20","op":"deploy","tick":"gwei","max":"21000000","lim":"1000"}"#;

        // Deserialize JSON to struct
        let operation: Deploy =
            serde_json::from_str(json_data).expect("Failed to deserialize JSON");

        let serialized_json =
            serde_json::to_string(&operation).expect("Failed to serialize to JSON");

        // Test deserialization
        let expected_operation = Deploy {
            p: "erc-20".to_string(),
            tick: "gwei".to_string(),
            max: 21000000,
            lim: 1000,
        };

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
