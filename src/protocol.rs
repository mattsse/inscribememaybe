//! Inscription protocol types

use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// An inscription protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protocol(ProtocolKind);

impl fmt::Display for Protocol {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for Protocol {
    fn from(id: &'a str) -> Self {
        Self(ProtocolKind::from(id))
    }
}

impl<'a> From<&'a String> for Protocol {
    fn from(id: &'a String) -> Self {
        Self(ProtocolKind::from(id))
    }
}

impl From<String> for Protocol {
    fn from(id: String) -> Self {
        Self(ProtocolKind::from(id))
    }
}

impl From<ProtocolKind> for Protocol {
    fn from(kind: ProtocolKind) -> Self {
        Self(kind)
    }
}

impl From<NamedProtocol> for Protocol {
    fn from(kind: NamedProtocol) -> Self {
        Self(ProtocolKind::Named(kind))
    }
}

/// A protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtocolKind {
    /// A known protocol
    Named(NamedProtocol),
    /// Any other protocol
    Other(String),
}

impl<'a> From<&'a str> for ProtocolKind {
    fn from(id: &'a str) -> Self {
        if let Ok(kind) = NamedProtocol::from_str(id) {
            ProtocolKind::Named(kind)
        } else {
            ProtocolKind::Other(id.to_string())
        }
    }
}
impl<'a> From<&'a String> for ProtocolKind {
    fn from(id: &'a String) -> Self {
        id.as_str().into()
    }
}

impl From<String> for ProtocolKind {
    fn from(id: String) -> Self {
        if let Ok(kind) = NamedProtocol::from_str(id.as_ref()) {
            ProtocolKind::Named(kind)
        } else {
            ProtocolKind::Other(id)
        }
    }
}

impl fmt::Display for ProtocolKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolKind::Named(chain) => chain.fmt(f),
            ProtocolKind::Other(id) => id.fmt(f),
        }
    }
}

impl From<NamedProtocol> for ProtocolKind {
    fn from(kind: NamedProtocol) -> Self {
        ProtocolKind::Named(kind)
    }
}

/// A known protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::IntoStaticStr)] // Into<&'static str>, AsRef<str>
#[derive(strum::EnumVariantNames)] // NamedProtocol::VARIANTS
#[derive(strum::EnumString)] // FromStr, TryFrom<&str>
#[derive(strum::EnumIter)] // NamedChain::iter
#[derive(strum::EnumCount)] // NamedChain::COUNT
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs, non_camel_case_types)]
pub enum NamedProtocol {
    Bsc_20,
    Asc_20,
    Prc_20,
    Zrc_20,
    Erc_20,
    Grc_20,
    Fair_20,
    Oprc_20,
    Osc_20,
    Brc_20,
    Frc_20,
    Nirc_20,
    Zsc_20,
    Vims_20,
    Era_20,
    Bnb_48,
    Gno_20,
    Terc_20,
    Nrc_20,
    Bep_20,
    Bnb_20,
    Cls_20,
    Base_20,
    Erc_cash,
    Bnbs_20,
    Ftm_20,
}

impl NamedProtocol {
    /// Returns the string representation of the protocol.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

impl fmt::Display for NamedProtocol {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl AsRef<str> for NamedProtocol {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_protocol() {
        let protocol = Protocol::from(NamedProtocol::Bsc_20);
        let s = serde_json::to_string(&protocol).unwrap();
        assert_eq!(s, r#""bsc-20""#);
    }
}
