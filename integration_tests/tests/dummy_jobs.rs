pub use swirl::Job;

use swirl::errors::PerformError;
use serde::*;

use crate::sync::Barrier;

#[derive(Serialize, Deserialize)]
/// A job which takes a barrier as its environment and calls wait on it before
/// succeeding
pub struct BarrierJob;

impl Job for BarrierJob {
    type Environment = Barrier;

    const JOB_TYPE: &'static str = "BarrierJob";

    fn perform(self, env: &Self::Environment) -> Result<(), PerformError> {
        env.wait();
        Ok(())
    }
}
