use std::collections::HashMap;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use rand::Rng;

use crate::actuator::FinalControlElement;
use crate::sensor::Sensor;

use super::{Buckets, MAX_QUANTITY};

/// NBuckets represents a fixed number set of buckets that randomly, monotonically increase in
/// fluid quantity.
/// TODO: Parameterize fill strategy?
pub struct NBuckets {
    data: HashMap<u64, u64>,
}

impl NBuckets {
    pub fn new(data: HashMap<u64, u64>) -> NBuckets {
        NBuckets { data }
    }

    fn get_bucket(&self, bucket: u64) -> Result<u64> {
        self.data
            .get(&bucket)
            .map(|quantity| *quantity)
            .ok_or(anyhow!("no bucket @ {}", bucket))
    }
}

impl Buckets for NBuckets {
    fn fill(&mut self) {
        let mut rng = rand::rng();

        for (_, value) in self.data.iter_mut() {
            let change = rng.random_range(0..=1);
            *value = value.saturating_add_signed(change);
        }
    }

    fn data(&self) -> Vec<(String, u64)> {
        self.data
            .iter()
            .sorted()
            .map(|(name, val)| (format!("B{}", name), *val))
            .collect()
    }
}

impl Sensor for NBuckets {
    fn buckets(&self) -> &HashMap<u64, u64> {
        &self.data
    }

    fn get_bucket_quantity(&self, bucket: u64) -> Result<u64> {
        self.get_bucket(bucket)
    }
}

impl FinalControlElement for NBuckets {
    fn transfer(&mut self, source: u64, destination: u64, amount: u64) -> Result<()> {
        let source_amount = self.get_bucket(source)?;
        let destination_amount = self.get_bucket(destination)?;
        if source_amount < amount {
            return Err(anyhow!(
                "transfer amount exceeds source amount ({} > {})",
                amount,
                source_amount
            ));
        }

        if destination_amount + amount > MAX_QUANTITY {
            return Err(anyhow!(
                "destination amount is too large for transfer ({} + {} > {})",
                destination_amount,
                amount,
                MAX_QUANTITY
            ));
        }

        let new_source_amount = source_amount - amount;
        let new_destination_amount = destination_amount + amount;

        self.data.insert(source, new_source_amount);
        self.data.insert(destination, new_destination_amount);

        Ok(())
    }

    fn add_bucket(&mut self) -> Result<u64> {
        Err(anyhow!("not implemented for type"))
    }
}
