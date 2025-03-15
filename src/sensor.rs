use std::collections::HashMap;

use anyhow::Result;

// TODO: We should have a dedicated type for bucket ID.
pub trait Sensor {
    fn buckets(&self) -> &HashMap<u64, u64>;
    fn get_smallest_bucket(&self) -> Option<(u64, u64)> {
        self.buckets()
            .iter()
            // Sort the tuples with the value of the bucket first, so only the smallest buckets are
            // returned. Break the tie by bucket ID.
            .map(|(bucket, value)| (value, bucket))
            .min()
            // Now flip it back to normal.
            .map(|(value, bucket)| (*bucket, *value))
    }
    fn get_largest_bucket(&self) -> Option<(u64, u64)> {
        // NOTE: See the implementation of get_smallest_bucket().
        self.buckets()
            .iter()
            // Sort the tuples with the value of the bucket first, so only the smallest buckets are
            // returned. Break the tie by bucket ID.
            .map(|(bucket, value)| (value, bucket))
            .max()
            // Now flip it back to normal.
            .map(|(value, bucket)| (*bucket, *value))
    }
    fn get_bucket_quantity(&self, bucket: u64) -> Result<u64>;
}
