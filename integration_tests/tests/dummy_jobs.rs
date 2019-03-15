pub use swirl::Job;

use serde::*;
use swirl::errors::PerformError;

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

swirl::register_job!(BarrierJob);
