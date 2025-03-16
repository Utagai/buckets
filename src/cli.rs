use std::{collections::HashMap, str::FromStr};

use clap::Parser;

use crate::buckets::BucketType;
use crate::policy::Policy;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Type of buckets to use.
    #[arg(short, long, value_enum, default_value_t = BucketType::NBuckets)]
    pub bucket_type: BucketType,

    /// Policy to apply.
    #[arg(short, long, value_enum, default_value_t = Policy::NoOp)]
    pub policy: Policy,

    /// Initial data in format "id1:value1,id2:value2,...".
    #[arg(short, long, value_parser = parse_initial_data, default_value = "1:45,2:72,3:38")]
    pub initial_data: HashMap<u64, u64>,

    /// Controller loop latency (ms).
    #[arg(short, long, default_value_t = 1000)]
    pub controller_latency: u64,

    /// Actuator loop latency (ms).
    #[arg(short, long, default_value_t = 1000)]
    pub actuator_latency: u64,

    /// Fill loop latency (ms).
    #[arg(short, long, default_value_t = 1000)]
    pub fill_latency: u64,
}

// Custom parser for the initial data
fn parse_initial_data(s: &str) -> Result<HashMap<u64, u64>, String> {
    let mut data = HashMap::new();

    if s.is_empty() {
        return Ok(data);
    }

    for pair in s.split(',') {
        let parts: Vec<&str> = pair.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid format for pair: {}", pair));
        }

        let id = u64::from_str(parts[0].trim()).map_err(|e| format!("Invalid ID: {}", e))?;
        let value = u64::from_str(parts[1].trim()).map_err(|e| format!("Invalid value: {}", e))?;

        data.insert(id, value);
    }

    Ok(data)
}
