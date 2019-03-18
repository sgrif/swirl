use assert_matches::assert_matches;
use diesel::prelude::*;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;
use swirl::schema::*;

use crate::dummy_jobs::*;
use crate::sync::Barrier;
use crate::test_guard::TestGuard;

#[test]
fn run_all_pending_jobs_returns_when_all_jobs_enqueued() {
    let barrier = Barrier::new(3);
    let runner = TestGuard::barrier_runner(barrier.clone());
    let conn = runner.connection_pool().get().unwrap();
    BarrierJob.enqueue(&conn).unwrap();
    BarrierJob.enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();

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
}

#[test]
fn assert_no_failed_jobs_blocks_until_all_queued_jobs_are_finished() {
    let barrier = Barrier::new(3);
    let runner = TestGuard::barrier_runner(barrier.clone());
    let conn = runner.connection_pool().get().unwrap();
    BarrierJob.enqueue(&conn).unwrap();
    BarrierJob.enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();

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

    runner.assert_no_failed_jobs().unwrap();
    send.send(1).unwrap();
    handle.join().unwrap();
}

#[test]
#[should_panic(expected = "3 jobs failed")]
fn assert_no_failed_jobs_panics_if_jobs_failed() {
    let runner = TestGuard::dummy_runner();
    let conn = runner.connection_pool().get().unwrap();
    FailureJob.enqueue(&conn).unwrap();
    FailureJob.enqueue(&conn).unwrap();
    FailureJob.enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();
    runner.assert_no_failed_jobs().unwrap();
}

#[test]
#[should_panic(expected = "2 jobs failed")]
fn panicking_jobs_are_caught_and_treated_as_failures() {
    let runner = TestGuard::dummy_runner();
    let conn = runner.connection_pool().get().unwrap();
    PanicJob.enqueue(&conn).unwrap();
    FailureJob.enqueue(&conn).unwrap();

    runner.run_all_pending_jobs().unwrap();
    runner.assert_no_failed_jobs().unwrap();
}

#[test]
fn run_all_pending_jobs_errs_if_jobs_dont_start_in_timeout() {
    let barrier = Barrier::new(2);
    // A runner with 1 thread where all jobs will hang indefinitely.
    // The second job will never start.
    let runner = TestGuard::builder(barrier.clone())
        .thread_count(1)
        .job_start_timeout(Duration::from_millis(50))
        .build();
    let conn = runner.connection_pool().get().unwrap();
    BarrierJob.enqueue(&conn).unwrap();
    BarrierJob.enqueue(&conn).unwrap();

    let run_result = runner.run_all_pending_jobs();
    assert_matches!(run_result, Err(swirl::FetchError::NoMessageReceived));

    // Make sure the jobs actually run so we don't panic on drop
    barrier.wait();
    barrier.wait();
    runner.assert_no_failed_jobs().unwrap();
}
