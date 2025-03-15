//! This file contains the implementation for the "actuator".
//! In control theory, an actuator is an entity in the control system responsible for taking
//! control signal (from the "controller") and undertaking actions suggested by it. These actions
//! may be anything, ranging from doing nothing, to taking corrective actions. The actuator is
//! "dumb", in that it knows how to do the actions and will do then when instructed by the control
//! signal, but has absolutely no idea about the original sensor data that encouraged this action.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum Action {
    Transfer {
        source: u64,
        destination: u64,
        amount: u64,
    },
    NoAction,
}

pub trait BucketsSystem {
    // TODO: Error type is bad.
    fn transfer(&mut self, source: u64, destination: u64, amount: u64) -> Result<()>;
    fn add_bucket(&mut self) -> Result<u64>;
}

pub(crate) struct Actuator<B: BucketsSystem> {
    buckets: Arc<Mutex<B>>,
    control_signal_rx: Receiver<Action>,
}

impl<B: BucketsSystem> Actuator<B> {
    pub fn new(buckets: Arc<Mutex<B>>, control_signal_rx: Receiver<Action>) -> Self {
        Actuator {
            buckets,
            control_signal_rx,
        }
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
        let maybe_action = self.control_signal_rx.recv().await;
        match maybe_action {
            Some(Action::Transfer {
                source,
                destination,
                amount,
            }) => {
                eprintln!("transfering {} from {} to {}", amount, source, destination);
                Ok(())
            }
            Some(Action::NoAction) => Ok(()),
            None => return Ok(()),
        }
    }
}
