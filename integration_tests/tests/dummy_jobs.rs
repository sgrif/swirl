pub use swirl::Job;

use serde::*;
use swirl::errors::PerformError;

use crate::sync::Barrier;

/// A job which takes a barrier as its environment and calls wait on it before
/// succeeding
#[derive(Serialize, Deserialize)]
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

/// A job which always fails
#[derive(Serialize, Deserialize)]
pub struct FailureJob;

impl Job for FailureJob {
    type Environment = ();

    const JOB_TYPE: &'static str = "FailureJob";

    fn perform(self, _: &Self::Environment) -> Result<(), PerformError> {
        Err("failed".into())
    }
}

swirl::register_job!(FailureJob);
#[derive(Serialize, Deserialize)]
/// A job which panics
pub struct PanicJob;

impl Job for PanicJob {
    type Environment = ();

    const JOB_TYPE: &'static str = "PanicJob";

    fn perform(self, _: &Self::Environment) -> Result<(), PerformError> {
        panic!()
    }
}

swirl::register_job!(PanicJob);
