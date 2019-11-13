use diesel::prelude::*;
use std::error::Error;
use std::time::Instant;
use swirl::*;

#[swirl::background_job]
fn dummy_job() -> Result<(), PerformError> {
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let database_url = dotenv::var("DATABASE_URL")?;
    println!("Enqueuing 100k jobs");
    let runner = Runner::builder(database_url, ()).build();
    enqueue_jobs(&*runner.connection_pool().get()?).unwrap();
    println!("Running jobs");
    let started = Instant::now();

    runner.run_all_pending_jobs()?;
    runner.check_for_failed_jobs()?;

    let elapsed = started.elapsed();
    println!("Ran 100k jobs in {} seconds", elapsed.as_secs());

    Ok(())
}

fn enqueue_jobs(conn: &PgConnection) -> Result<(), EnqueueError> {
    use diesel::sql_query;
    sql_query("TRUNCATE TABLE background_jobs;").execute(conn)?;
    for _ in 0..100_000 {
        dummy_job().enqueue(conn)?;
    }
    Ok(())
}
