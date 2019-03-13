use diesel::prelude::*;
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
#[ignore]
fn wait_for_jobs_blocks_until_all_queued_jobs_are_finished() {
    panic!("pending")
}
