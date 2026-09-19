use chia_wallet_sdk::utils::Address;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::normalized_non_empty_string;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppDonation {
    address: String,
}

impl SageAppDonation {
    pub fn new(address: impl Into<String>) -> anyhow::Result<Self> {
        let address = normalized_non_empty_string(address, "donation address")?;

        let decoded = Address::decode(&address)
            .map_err(|_| anyhow::anyhow!("invalid donation address format"))?;

        if decoded.prefix != "xch" {
            anyhow::bail!("invalid donation address format");
        }

        Ok(Self { address })
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_mainnet_address() {
        SageAppDonation::new("xch10hwpheqwv4m4dzuqc5hp8se3p0chkrkvmtf8m6d830wfagg9p22sxd82v7")
            .unwrap();
    }

    #[test]
    fn rejects_testnet_address() {
        let err =
            SageAppDonation::new("txch1juv3a2rlelg7k52g6g7kze4j7rrxlgx88uu70d7x4p0kwx49znyq4xhjyy")
                .unwrap_err();

        assert_eq!(err.to_string(), "invalid donation address format");
    }

    #[test]
    fn rejects_invalid_mainnet_address() {
        let err = SageAppDonation::new("xch1not-a-valid-address").unwrap_err();

        assert_eq!(err.to_string(), "invalid donation address format");
    }
}
