use diesel::result::Error as DieselError;

use super::channel;
use crate::db::DieselPool;

pub type EventSender<Pool> = channel::Sender<Event<Pool>>;

pub enum Event<Pool: DieselPool> {
    Working,
    NoJobAvailable,
    ErrorLoadingJob(DieselError),
    FailedToAcquireConnection(Pool::Error),
}

use std::fmt;

impl<Pool: DieselPool> fmt::Debug for Event<Pool> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::Working => f.debug_struct("Working").finish(),
            Event::NoJobAvailable => f.debug_struct("NoJobAvailable").finish(),
            Event::ErrorLoadingJob(e) => f.debug_tuple("ErrorLoadingJob").field(e).finish(),
            Event::FailedToAcquireConnection(e) => {
                f.debug_tuple("FailedToAcquireConnection").field(e).finish()
            }
        }
    }
}
