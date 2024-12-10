// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::common::{Address, Amount, QuoteHash};
use crate::contract::payment_vault::handler::PaymentVaultHandler;
use crate::quoting_metrics::QuotingMetrics;
use crate::utils::http_provider;
use crate::{contract, Network};
use alloy::transports::{RpcError, TransportErrorKind};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RpcError(#[from] RpcError<TransportErrorKind>),
    #[error(transparent)]
    PaymentVaultError(#[from] contract::payment_vault::error::Error),
    #[error("Payment missing")]
    PaymentMissing,
}

/// Verify if a data payment is confirmed.
pub async fn verify_data_payment(
    network: &Network,
    my_quote_hashes: Vec<QuoteHash>, // TODO @mick hashes the node owns so it knows how much it received from them
    payment: Vec<(QuoteHash, QuotingMetrics, Address)>,
) -> Result<Amount, Error> {
    let provider = http_provider(network.rpc_url().clone());
    let payment_vault = PaymentVaultHandler::new(*network.data_payments_address(), provider);

    // NB TODO @mick remove tmp loop and support verification of the whole payment at once
    let mut is_paid = true;
    for (quote_hash, quoting_metrics, reward_addr) in payment {
        is_paid = payment_vault
            .verify_payment(quoting_metrics, (quote_hash, reward_addr, Amount::ZERO))
            .await?;
    }

    let amount_paid = Amount::ZERO; // NB TODO @mick we need to get the amount paid from the contract

    if is_paid {
        Ok(amount_paid)
    } else {
        Err(Error::PaymentMissing)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Address;
    use crate::quoting_metrics::QuotingMetrics;
    use crate::transaction::verify_data_payment;
    use crate::Network;
    use alloy::hex::FromHex;
    use alloy::primitives::b256;

    #[tokio::test]
    async fn test_verify_data_payment() {
        let network = Network::ArbitrumOne;

        let quote_hash = b256!("EBD943C38C0422901D4CF22E677DD95F2591CA8D6EBFEA8BAF1BFE9FF5506ECE"); // DevSkim: ignore DS173237
        let reward_address = Address::from_hex("8AB15A43305854e4AE4E6FBEa0CD1CC0AB4ecB2A").unwrap(); // DevSkim: ignore DS173237

        let result = verify_data_payment(
            &network,
            vec![(quote_hash, QuotingMetrics::default(), reward_address)],
        )
        .await;

        assert!(result.is_ok(), "Error: {:?}", result.err());
    }
}
