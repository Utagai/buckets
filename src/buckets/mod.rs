pub mod n_buckets;

// TODO: This should be a parameter.
pub const MAX_QUANTITY: u64 = 100;

pub(crate) trait Buckets {
    fn fill(&mut self);
    fn data(&self) -> Vec<(String, u64)>;
}
