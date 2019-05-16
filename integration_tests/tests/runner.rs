use assert_matches::assert_matches;
use diesel::prelude::*;
use failure::Fallible;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;
use swirl::schema::*;
use swirl::JobsFailed;

use crate::dummy_jobs::*;
use crate::sync::Barrier;
use crate::test_guard::TestGuard;

#[test]
fn run_all_pending_jobs_returns_when_all_jobs_enqueued() -> Fallible<()> {
    let barrier = Barrier::new(3);
    let runner = TestGuard::runner(barrier.clone());
    let conn = runner.connection_pool().get()?;
    barrier_job().enqueue(&conn)?;
    barrier_job().enqueue(&conn)?;

    runner.run_all_pending_jobs()?;

    let queued_job_count = background_jobs::table.count().get_result(&conn);
    let unlocked_job_count = background_jobs::table
        .select(background_jobs::id)
        .for_update()
        .skip_locked()
        .load::<i64>(&conn)
        .map(|v| v.len());

    assert_eq!(Ok(2), queued_job_count);
    assert_eq!(Ok(0), unlocked_job_count);

    barrier.wait();
    Ok(())
}

#[test]
fn check_for_failed_jobs_blocks_until_all_queued_jobs_are_finished() -> Fallible<()> {
    let barrier = Barrier::new(3);
    let runner = TestGuard::runner(barrier.clone());
    let conn = runner.connection_pool().get()?;
    barrier_job().enqueue(&conn)?;
    barrier_job().enqueue(&conn)?;

    runner.run_all_pending_jobs()?;

    let (send, recv) = sync_channel(0);
    let handle = thread::spawn(move || {
        let wait = Duration::from_millis(100);
        assert!(
            recv.recv_timeout(wait).is_err(),
            "wait_for_jobs returned before jobs finished"
        );

        barrier.wait();

        assert!(recv.recv().is_ok(), "wait_for_jobs didn't return");
    });

    runner.check_for_failed_jobs()?;
    send.send(1)?;
    handle.join().unwrap();
    Ok(())
}

#[test]
fn check_for_failed_jobs_panics_if_jobs_failed() -> Fallible<()> {
    let runner = TestGuard::dummy_runner();
    let conn = runner.connection_pool().get()?;
    failure_job().enqueue(&conn)?;
    failure_job().enqueue(&conn)?;
    failure_job().enqueue(&conn)?;

    runner.run_all_pending_jobs()?;
    assert_eq!(Err(JobsFailed(3)), runner.check_for_failed_jobs());
    Ok(())
}

#[test]
fn panicking_jobs_are_caught_and_treated_as_failures() -> Fallible<()> {
    let runner = TestGuard::dummy_runner();
    let conn = runner.connection_pool().get()?;
    panic_job().enqueue(&conn)?;
    failure_job().enqueue(&conn)?;

    runner.run_all_pending_jobs()?;
    assert_eq!(Err(JobsFailed(2)), runner.check_for_failed_jobs());
    Ok(())
}

#[test]
fn run_all_pending_jobs_errs_if_jobs_dont_start_in_timeout() -> Fallible<()> {
    let barrier = Barrier::new(2);
    // A runner with 1 thread where all jobs will hang indefinitely.
    // The second job will never start.
    let runner = TestGuard::builder(barrier.clone())
        .thread_count(1)
        .job_start_timeout(Duration::from_millis(50))
        .build();
    let conn = runner.connection_pool().get()?;
    barrier_job().enqueue(&conn)?;
    barrier_job().enqueue(&conn)?;

    let run_result = runner.run_all_pending_jobs();
    assert_matches!(run_result, Err(swirl::FetchError::NoMessageReceived));

    // Make sure the jobs actually run so we don't panic on drop
    barrier.wait();
    barrier.wait();
    runner.check_for_failed_jobs()?;
    Ok(())
}

#[test]
fn jobs_failing_to_load_doesnt_panic_threads() {
    let runner = TestGuard::with_db_pool_size((), 1).thread_count(1).build();

    {
        let conn = runner.connection_pool().get().unwrap();
        failure_job().enqueue(&conn).unwrap();
        // Since jobs are loaded with `SELECT FOR UPDATE`, it will always fail in
        // read-only mode
        diesel::sql_query("SET default_transaction_read_only = 't'")
            .execute(&conn)
            .unwrap();
    }

    let run_result = runner.run_all_pending_jobs();

    {
        let conn = runner.connection_pool().get().unwrap();
        diesel::sql_query("SET default_transaction_read_only = 'f'")
            .execute(&conn)
            .unwrap();
    }

    assert_matches!(run_result, Err(swirl::FetchError::FailedLoadingJob(_)));
    runner.assert_no_failed_jobs().unwrap();
}
