use rand::Rng;

pub struct Buckets {
    data: Vec<(String, u64)>,
}

impl Buckets {
    pub fn new(data: Vec<(String, u64)>) -> Buckets {
        Buckets { data }
    }

    fn fill(&mut self) {
        let mut rng = rand::rng();

        for (_, value) in self.data.iter_mut() {
            let change = rng.random_range(0..=1);
            *value = value.saturating_add_signed(change);
        }
    }

    pub fn data(&self) -> Vec<(String, u64)> {
        self.data.clone()
    }

    pub async fn tick(&mut self) {
        self.fill();
    }
}
