use std::fmt::Display;

use clap::ValueEnum;

pub mod n_buckets;

// TODO: This should be a parameter.
pub const MAX_QUANTITY: u64 = 100;

pub(crate) trait Buckets {
    fn fill(&mut self) -> (u64, u64, u64);
    fn data(&self) -> Vec<(String, u64)>;
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum BucketType {
    NBuckets,
}

impl Display for BucketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BucketType::NBuckets => write!(f, "NBuckets"),
        }
    }
}
