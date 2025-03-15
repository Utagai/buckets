//! A policy implements a particular control strategy for a controller.
//! A policy implementation takes signal from a sensor and emits a control signal based on it.
use std::collections::HashMap;
use std::fmt::Display;

use tokio::sync::MutexGuard;

use crate::actuator::Action;
use crate::controller::Sensor;

pub enum Policy {
    Spread,
}

impl Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Policy::Spread => {
                write!(f, "Spread")
            }
        }
    }
}

impl Policy {
    pub(crate) fn analyze<S: Sensor>(&self, sensor: MutexGuard<S>) -> Action {
        eprintln!("analyzing sensor data for policy: {}", self);
        match self {
            Policy::Spread => {
                eprintln!(
                    "analyzing sensor data for Spread policy: {:?}",
                    buckets_signal
                );
                Action::Transfer {
                    source: 1,
                    destination: 2,
                    amount: 10,
                }
            }
        }
    }
}
