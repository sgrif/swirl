use antidote::{Mutex, MutexGuard};
use diesel::prelude::*;
use diesel::r2d2;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use swirl::{Builder, Runner};

use crate::db::*;
use crate::sync::Barrier;
use crate::util::*;

lazy_static::lazy_static! {
    // Since these tests deal with behavior concerning multiple connections
    // running concurrently, they have to run outside of a transaction.
    // Therefore we can't run more than one at a time.
    //
    // Rather than forcing the whole suite to be run with `--test-threads 1`,
    // we just lock these tests instead.
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

pub struct TestGuard<'a, Env: 'static> {
    runner: Runner<Env, DieselPool>,
    _lock: MutexGuard<'a, ()>,
}

impl<'a, Env> TestGuard<'a, Env> {
    pub fn builder(env: Env) -> GuardBuilder<Env> {
        use dotenv;

        let database_url =
            dotenv::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests");
        let manager = r2d2::ConnectionManager::new(database_url);
        let pool = pool_builder().build_unchecked(manager);

        let builder = Runner::builder(pool, env);

        GuardBuilder { builder }
    }
}

impl<'a> TestGuard<'a, ()> {
    pub fn dummy_runner() -> Self {
        Self::builder(()).build()
    }
}

impl<'a> TestGuard<'a, Barrier> {
    pub fn barrier_runner(env: Barrier) -> Self {
        Self::builder(env).build()
    }
}

pub struct GuardBuilder<Env: 'static> {
    builder: Builder<Env, DieselPool>,
}

impl<Env> GuardBuilder<Env> {
    pub fn thread_count(mut self, count: usize) -> Self {
        self.builder = self.builder.thread_count(count);
        self
    }

    pub fn job_start_timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.job_start_timeout(timeout);
        self
    }

    pub fn build<'a>(self) -> TestGuard<'a, Env> {
        TestGuard {
            _lock: TEST_MUTEX.lock(),
            runner: self.builder.build(),
        }
    }
}

impl<'a, Env> Deref for TestGuard<'a, Env> {
    type Target = Runner<Env, DieselPool>;

    fn deref(&self) -> &Self::Target {
        &self.runner
    }
}

impl<'a, Env> DerefMut for TestGuard<'a, Env> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runner
    }
}

impl<'a, Env> Drop for TestGuard<'a, Env> {
    fn drop(&mut self) {
        let conn = self.runner.connection_pool().get().unwrap();
        ::diesel::sql_query("TRUNCATE TABLE background_jobs")
            .execute(&conn)
            .unwrap_from_drop();
    }
}
