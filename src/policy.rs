//! A policy implements a particular control strategy for a controller.
//! A policy implementation takes signal from a sensor and emits a control signal based on it.
use std::fmt::Display;

use tokio::sync::MutexGuard;

use clap::ValueEnum;

use crate::actuator::Action;
use crate::sensor::Sensor;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum Policy {
    Spread,
    NoOp,
}

impl Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Policy::Spread => {
                write!(f, "Spread")
            }
            Policy::NoOp => {
                write!(f, "NoOp")
            }
        }
    }
}

impl Policy {
    pub(crate) fn analyze<S: Sensor>(&self, sensor: MutexGuard<S>) -> Action {
        eprintln!("analyzing sensor data for policy: {}", self);
        match self {
            Policy::Spread => {
                // NOTE: This implementation is actually a bit inefficient, since we could
                // technically try to fix the imbalance immediately instead of doing a single tiny
                // transfer each time.
                let min_bucket = sensor.get_smallest_bucket();
                let max_bucket = sensor.get_largest_bucket();
                if min_bucket == max_bucket {
                    // All buckets are equal, nothing to do!
                    return Action::NoAction;
                }
                if let Some((min_bucket, min_qty)) = min_bucket {
                    if let Some((max_bucket, max_qty)) = max_bucket {
                        let diff = max_qty - min_qty;
                        let transfer_amount = diff / 2;
                        return Action::Transfer {
                            source: max_bucket,
                            destination: min_bucket,
                            amount: transfer_amount,
                        };
                    }
                }

                eprintln!("unreachable?: could not unpack min/max bucket information");
                Action::NoAction
            }
            Policy::NoOp => Action::NoAction,
        }
    }
}
