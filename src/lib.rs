//! # Features
//! `web3_`: Implements `GasPriceEstimating` for `Web3`.

#[cfg(feature = "tokio_")]
pub mod blocknative;
#[cfg(feature = "web3_")]
pub mod eth_node;
pub mod ethgasstation;
pub mod gasnow;
#[cfg(feature = "tokio_")]
pub mod gasnow_websocket;
pub mod gnosis_safe;
mod linear_interpolation;
pub mod priority;

pub use ethgasstation::EthGasStation;
pub use gasnow::GasNowGasStation;
#[cfg(feature = "tokio_")]
pub use gasnow_websocket::GasNowWebSocketGasStation;
pub use gnosis_safe::GnosisSafeGasStation;
pub use priority::PriorityGasPriceEstimating;

use anyhow::Result;
use serde::de::DeserializeOwned;
use std::time::Duration;

pub const DEFAULT_GAS_LIMIT: f64 = 21000.0;
pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_secs(30);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GasPrice1559 {
    base_fee_per_gas: f64,
    max_fee_per_gas: f64,
    max_priority_fee_per_gas: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GasPrice {
    legacy: f64,
    eip1559: Option<GasPrice1559>,
}

impl GasPrice {
    pub fn estimate_gas_price(&self) -> f64 {
        if let Some(gas_price) = &self.eip1559 {
            match gas_price
                .max_fee_per_gas
                .partial_cmp(&(gas_price.max_priority_fee_per_gas + gas_price.base_fee_per_gas))
            {
                Some(ordering) => match ordering {
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                        gas_price.max_fee_per_gas
                    }
                    std::cmp::Ordering::Greater => {
                        gas_price.max_priority_fee_per_gas + gas_price.base_fee_per_gas
                    }
                },
                None => gas_price.max_fee_per_gas,
            }
        } else {
            self.legacy
        }
    }

    pub fn bump(self, factor: f64) -> Self {
        Self {
            legacy: self.legacy * factor,
            eip1559: match self.eip1559 {
                Some(x) => Some(GasPrice1559 {
                    base_fee_per_gas: x.base_fee_per_gas,
                    max_fee_per_gas: x.max_fee_per_gas * factor,
                    max_priority_fee_per_gas: x.max_priority_fee_per_gas,
                }),
                None => None,
            },
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait GasPriceEstimating: Send + Sync {
    /// Estimate the gas price for a transaction to be mined "quickly".
    async fn estimate(&self) -> Result<GasPrice> {
        self.estimate_with_limits(DEFAULT_GAS_LIMIT, DEFAULT_TIME_LIMIT)
            .await
    }
    /// Estimate the gas price for a transaction that uses <gas> to be mined within <time_limit>.
    async fn estimate_with_limits(&self, gas_limit: f64, time_limit: Duration) -> Result<GasPrice>;
}

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
        header: http::header::HeaderMap,
    ) -> Result<T>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    #[derive(Default)]
    pub struct TestTransport {}

    #[async_trait::async_trait]
    impl Transport for TestTransport {
        async fn get_json<T: DeserializeOwned>(
            &self,
            url: &str,
            header: http::header::HeaderMap,
        ) -> Result<T> {
            let json = reqwest::Client::new()
                .get(url)
                .headers(header)
                .send()
                .await?
                .text()
                .await?;

            Ok(serde_json::from_str(&json)?)
        }
    }

    pub trait FutureWaitExt: Future + Sized {
        fn wait(self) -> Self::Output {
            futures::executor::block_on(self)
        }
    }
    impl<F> FutureWaitExt for F where F: Future {}
}
