use std::error::Error;

/// An error occurred queueing the job
pub type EnqueueError = Box<dyn Error>;

/// An error occurred performing the job
pub type PerformError = Box<dyn Error>;
