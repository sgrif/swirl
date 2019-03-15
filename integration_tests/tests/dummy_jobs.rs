pub use swirl::Job;

use swirl::errors::PerformError;

use crate::sync::Barrier;

/// A job which takes a barrier as its environment and calls wait on it before
/// succeeding
#[swirl::background_job]
pub fn barrier_job(env: &Barrier) -> Result<(), PerformError> {
    env.wait();
    Ok(())
}

/// A job which always fails
#[swirl::background_job]
pub fn failure_job() -> Result<(), PerformError> {
    Err("failed".into())
}

#[swirl::background_job]
/// A job which panics
pub fn panic_job() -> Result<(), PerformError> {
    panic!()
}
