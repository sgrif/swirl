use diesel::result::Error as DieselError;
use std::error::Error;
use std::fmt;

use crate::db::DieselPool;

/// An error occurred queueing the job
pub type EnqueueError = Box<dyn Error + Send + Sync>;

/// An error occurred performing the job
pub type PerformError = Box<dyn Error>;

/// An error occurred while attempting to fetch jobs from the queue
pub enum FetchError<Pool: DieselPool> {
    /// We could not acquire a database connection from the pool.
    ///
    /// Either the connection pool is too small, or new connections cannot be
    /// established.
    NoDatabaseConnection(Pool::Error),

    /// Could not execute the query to load a job from the database.
    FailedLoadingJob(DieselError),

    /// No message was received from the worker thread.
    ///
    /// Either the thread pool is too small, or jobs have hung indefinitely
    NoMessageReceived,
}

impl<Pool: DieselPool> fmt::Debug for FetchError<Pool> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchError::NoDatabaseConnection(e) => {
                f.debug_tuple("NoDatabaseConnection").field(e).finish()
            }
            FetchError::FailedLoadingJob(e) => f.debug_tuple("FailedLoadingJob").field(e).finish(),
            FetchError::NoMessageReceived => f.debug_struct("NoMessageReceived").finish(),
        }
    }
}

impl<Pool: DieselPool> fmt::Display for FetchError<Pool> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchError::NoDatabaseConnection(e) => {
                write!(f, "Timed out acquiring a database connection. ")?;
                write!(f, "Try increasing the connection pool size: ")?;
                write!(f, "{}", e)?;
            }
            FetchError::FailedLoadingJob(e) => {
                write!(f, "An error occurred loading a job from the database: ")?;
                write!(f, "{}", e)?;
            }
            FetchError::NoMessageReceived => {
                write!(f, "No message was received from the worker thread. ")?;
                write!(f, "Try increasing the thread pool size or timeout period.")?;
            }
        }
        Ok(())
    }
}

impl<Pool: DieselPool> Error for FetchError<Pool> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FetchError::NoDatabaseConnection(e) => Some(e),
            FetchError::FailedLoadingJob(e) => Some(e),
            FetchError::NoMessageReceived => None,
        }
    }
}
