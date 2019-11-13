#![allow(clippy::new_without_default)] // https://github.com/rust-lang/rust-clippy/issues/3632

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::errors::PerformError;
use crate::Job;

#[derive(Default)]
#[allow(missing_debug_implementations)] // Can't derive debug
/// A registry of background jobs, used to map job types to concrete perform
/// functions at runtime.
pub struct Registry<Env> {
    jobs: HashMap<&'static str, JobVTable>,
    _marker: PhantomData<Env>,
}

impl<Env: 'static> Registry<Env> {
    /// Loads the registry from all invocations of [`register_job!`] for this
    /// environment type
    pub fn load() -> Self {
        let jobs = inventory::iter::<JobVTable>
            .into_iter()
            .filter(|vtable| vtable.env_type == TypeId::of::<Env>())
            .map(|&vtable| (vtable.job_type, vtable))
            .collect();

        Self {
            jobs: jobs,
            _marker: PhantomData,
        }
    }

    /// Get the perform function for a given job type
    pub fn get(&self, job_type: &str) -> Option<PerformJob<Env>> {
        self.jobs.get(job_type).map(|&vtable| PerformJob {
            vtable,
            _marker: PhantomData,
        })
    }
}

/// Register a job to be run by swirl. This must be called for any
/// implementors of [`swirl::Job`]
#[macro_export]
macro_rules! register_job {
    ($job_ty: ty) => {
        $crate::inventory::submit! {
            #![crate = swirl]
            swirl::JobVTable::from_job::<$job_ty>()
        }
    };
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct JobVTable {
    env_type: TypeId,
    job_type: &'static str,
    perform: fn(serde_json::Value, &dyn Any) -> Result<(), PerformError>,
}

inventory::collect!(JobVTable);

impl JobVTable {
    pub fn from_job<T: Job>() -> Self {
        Self {
            env_type: TypeId::of::<T::Environment>(),
            job_type: T::JOB_TYPE,
            perform: perform_job::<T>,
        }
    }
}

fn perform_job<T: Job>(data: serde_json::Value, env: &dyn Any) -> Result<(), PerformError> {
    let environment = env.downcast_ref().ok_or_else::<PerformError, _>(|| {
        "Incorrect environment type. This should never happen. \
         Please open an issue at https://github.com/sgrif/swirl/issues/new"
            .into()
    })?;
    let data = serde_json::from_value(data)?;
    T::perform(data, environment)
}

pub struct PerformJob<Env> {
    vtable: JobVTable,
    _marker: PhantomData<Env>,
}

impl<Env: 'static> PerformJob<Env> {
    pub fn perform(&self, data: serde_json::Value, env: &Env) -> Result<(), PerformError> {
        let perform_fn = self.vtable.perform;
        perform_fn(data, env)
    }
}
