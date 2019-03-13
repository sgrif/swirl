use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::{Arc, Barrier as StdBarrier, BarrierWaitResult};

#[derive(Clone)]
pub struct Barrier {
    inner: Arc<StdBarrier>,
}

impl Barrier {
    pub fn new(n: usize) -> Self {
        Self {
            inner: Arc::new(StdBarrier::new(n)),
        }
    }

    pub fn wait(&self) -> BarrierWaitResult {
        self.inner.wait()
    }
}

impl UnwindSafe for Barrier {}
impl RefUnwindSafe for Barrier {}
