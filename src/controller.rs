//! This file contains the implementation for the "controller".
//! In control theory, a controller is an entity in the control system responsible for taking
//! sensor data (known sometimes more formally as the "signal") and produces a secondary signal,
//! called the "control signal". The signal typically refers to things like sensor data or software
//! metrics. The control signal is a fancy way of saying "plan" or "action". The control signal is
//! used by the actuator (actuator.rs) to know what actions to take in order to correct the system
//! under control.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::actuator::{Action, Actuator};
use crate::policy::Policy;

pub trait Sensor {
    fn buckets(&self) -> HashMap<u64, u64>;
    fn get_bucket_quantity(&self, bucket: u64) -> Result<u64>;
}

pub struct Controller<S: Sensor> {
    policy: Policy,
    sensor: Arc<Mutex<S>>,
    control_signal_tx: Sender<Action>,
}

impl<S: Sensor> Controller<S> {
    pub fn new(policy: Policy, sensor: Arc<Mutex<S>>, control_signal_tx: Sender<Action>) -> Self {
        Controller {
            policy,
            sensor,
            control_signal_tx,
        }
    }
    pub async fn run(&self) -> Result<()> {
        let sensor = self.sensor.lock().await;
        let action = self.policy.analyze(sensor);
        self.control_signal_tx.send(action).await?;
        Ok(())
    }
}
