/// Gas price received from the gas price estimators.

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
/// Main gas price structure.
/// Provide estimated gas prices for both legacy and eip1559 transactions.
pub struct EstimatedGasPrice {
    // Estimated gas price for legacy type of transactions.
    pub legacy: f64,
    // Estimated gas price for 1559 type of transactions. Optional because not all gas estimators support 1559.
    pub eip1559: Option<GasPrice1559>,
}

impl EstimatedGasPrice {
    // Estimate the gas price based on the current network conditions (base_fee_per_gas)
    // Beware that gas price for mined transaction could be different from estimated value in case of 1559 tx
    // (because base_fee_per_gas can change between estimation and mining the tx).
    pub fn estimate(&self) -> f64 {
        if let Some(gas_price) = &self.eip1559 {
            std::cmp::min_by(
                gas_price.max_fee_per_gas,
                gas_price.max_priority_fee_per_gas + gas_price.base_fee_per_gas,
                |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            )
        } else {
            self.legacy
        }
    }

    // Maximum gas price willing to pay for the transaction.
    pub fn cap(&self) -> f64 {
        if let Some(gas_price) = &self.eip1559 {
            gas_price.max_fee_per_gas
        } else {
            self.legacy
        }
    }

    // Bump maximum gas price by factor.
    pub fn bump_cap(self, factor: f64) -> Self {
        Self {
            legacy: self.legacy * factor,
            eip1559: self.eip1559.and_then(|x| Some(x.bump_cap(factor))),
        }
    }

    // Ceil maximum gas price (since its defined as float).
    pub fn ceil_cap(self) -> Self {
        Self {
            legacy: self.legacy.ceil(),
            eip1559: self.eip1559.and_then(|x| Some(x.ceil_cap())),
        }
    }

    // If current cap if higher then the input, set to input.
    pub fn limit_cap(self, cap: f64) -> Self {
        Self {
            legacy: self.legacy.min(cap),
            eip1559: self.eip1559.and_then(|x| Some(x.limit_cap(cap))),
        }
    }
}

/// Gas price structure for 1559 transactions.
/// Contains base_fee_per_gas as an essential part of the gas price estimation.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct GasPrice1559 {
    // Estimated base fee for the pending block (block currently being mined)
    pub base_fee_per_gas: f64,
    // Maximum gas price willing to pay for the transaction.
    pub max_fee_per_gas: f64,
    // Priority fee used to incentivize miners to include the tx in case of network congestion.
    pub max_priority_fee_per_gas: f64,
}

impl GasPrice1559 {
    // Bump maximum gas price by factor.
    pub fn bump_cap(self, factor: f64) -> Self {
        Self {
            base_fee_per_gas: self.base_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas * factor,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
        }
    }

    // Ceil maximum gas price (since its defined as float).
    pub fn ceil_cap(self) -> Self {
        Self {
            base_fee_per_gas: self.base_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas.ceil(),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
        }
    }

    // If current cap if higher then the input, set to input.
    pub fn limit_cap(self, cap: f64) -> Self {
        Self {
            base_fee_per_gas: self.base_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas.min(cap),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
        }
    }
}

#[cfg(test)]
mod tests {
    // todo
}
