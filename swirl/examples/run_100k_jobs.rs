use diesel::prelude::*;
use diesel::r2d2;
use std::error::Error;
use std::time::Instant;
use swirl::*;

#[swirl::background_job]
fn dummy_job() -> Result<(), PerformError> {
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let database_url = dotenv::var("DATABASE_URL")?;
    let num_cpus = num_cpus::get();
    let connection_manager = r2d2::ConnectionManager::new(database_url);
    let connection_pool = r2d2::Pool::builder()
        .max_size(num_cpus as u32)
        .build(connection_manager)?;
    println!("Enqueuing 100k jobs");
    enqueue_jobs(&*connection_pool.get()?).unwrap();
    let runner = Runner::builder(connection_pool, ()).build();
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
