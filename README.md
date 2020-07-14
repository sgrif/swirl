Swirl
=====

A simple, efficient background work queue for Rust
--------------------------------------------------

Swirl is a background work queue built on Diesel and PostgreSQL's row locking
features. It was extracted from [crates.io](crates.io), which uses it for
updating the index of the web server.

This library is still in its early stages, and has not yet reached 0.1 status.
We're using it successfully in production on crates.io today, but there are
still several things missing that you may want from a job queue.

## Getting Started

Swirl stores background jobs in your PostgreSQL 9.5+ database. As such, it has
migrations which need to be run. At the moment, this should be done by copying
our migrations directory into your own. This will be improved before the crate
is released.

Jobs in Swirl are defined as functions annotated with
`#[swirl::background_job]`, like so:

```rust
#[swirl::background_job]
fn resize_image(file_name: String, dimensions: Size) -> Result<(), swirl::PerformError> {
    // Do expensive computation that shouldn't be done on the web server
}
```

All arguments must implement `serde::Serialize` and `serde::DeserializeOwned`.
Jobs can also take a shared "environment" argument. This is a struct you define,
which can contain resources shared between jobs like a connection pool, or
application level configuration. For example:

```rust
struct Environment {
    file_server_private_key: String,
    http_client: http_lib::Client,
}

#[swirl::background_job]
fn resize_image(
    env: &Environment,
    file_name: String,
    dimensions: Size,
) -> Result<(), swirl::PerformError> {
    // Do expensive computation that shouldn't be done on the web server
}
```

Note that all jobs must use the same type for the environment.
Once a job is defined, it can be enqueued like so:

```rust
resize_image(file_name, dimensions).enqueue(&diesel_connection)?
```

You do not pass the environment when enqueuing jobs.
Jobs are run asynchronously by an instance of `swirl::Runner`. To construct
one, you must first pass it the job environment (this is `()` if your jobs don't
take an environment), and a Diesel connection pool (from `diesel::r2d2`).

```rust
let runner = Runner::builder(environment, connection_pool)
    .build();
```

At the time of writing, it is up to you to make sure your connection pool is
well configured for your runner. Your connection pool size should be at least as
big as the thread pool size (defaults to the number of CPUs on your machine), or
double that if your jobs require a database connection.

Once the runner is created, calling `run_all_pending_jobs` will continuously
saturate all available threads, attempting to run one job per thread at a time.
It will return `Ok(())` once at least one thread has reported there were no jobs
available to run, or an error if a job fails to start running. Note that this
function does not know or care if a job *completes* successfully, only if we
were successful at starting to do work. Typically this function should be called
in a loop:

```rust
loop {
    if let Err(e) = runner.run_all_pending_jobs() {
        // Something has gone seriously wrong. The database might be down,
        // or the thread pool may have died. We could just try again, or
        // perhaps rebuild the runner, or crash/restart the process.
    }
}
```

In situations where you have low job throughput, you can add a sleep to this
loop to wait some period of time before looking for more jobs.

When a job fails (by returning an error or panicking), it will be retried after
`1 ^ {retry_count}` minutes. If a job fails or an error occurs marking a job as
finsihed/failed, it will be logged to stderr. No output will be sent when jobs
are running successfully.

Swirl uses at least once semantics. This means that we guarantee all jobs are
successfully run to completion, but we do not guarantee that it will do so only
once, even if the job successfully returns `Ok(())`. Therefore, it is important
that all jobs are idempotent.

## Upcoming features

Planned features that are not yet implemented are:

- Automatic configuration of the DB connection pool
- Allowing jobs to take a database connection as an argument
  - If your jobs need a DB connection today, put the connection pool on your
    environment.
- More robust and configurable logging
- Configurable retry behavior
- Support for multiple queues with priority
- Less boilerplate in the job runner

## Code of conduct

Anyone who interacts with Swirl in any space, including but not limited to
this GitHub repository, must follow our [code of conduct](https://github.com/sgrif/swirl/blob/master/code_of_conduct.md).

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

### Contributing

Unless you explicitly state otherwise, any contribution you intentionally submit
for inclusion in the work, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.
