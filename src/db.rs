use diesel::PgConnection;

/// A connection pool for Diesel database connections
///
/// If you don't care about the details of connection pooling, or want to use
/// the r2d2 crate, you can enable the r2d2 feature on this crate and never
/// be concerned with this trait. If you want to use your own connection pool,
/// you can implement this trait manually.
pub trait DieselPool: Clone + Send {
    /// The smart pointer returned by this connection pool.
    type Connection: Deref<PgConnection>;

    /// The error type returned when a connection could not be retreived from
    /// the pool.
    type Error;

    /// Attempt to get a database connection from the pool. Errors if a
    /// connection could not be retrieved from the pool.
    ///
    /// The exact details of why an error would be returned will depend on
    /// the pool, but a reasonable implementation will return an error if:
    ///
    /// - A timeout was reached
    /// - An error occurred establishing a new connection
    fn get(&self) -> Result<Self::Connection, Self::Error>;
}

#[cfg(feature = "r2d2")]
mod r2d2_impl {
    use super::*;
    use diesel::r2d2;

    type ConnectionManager = r2d2::ConnectionManager<PgConnection>;

    impl DieselPool for r2d2::Pool<ConnectionManager>; {
        type Connection = r2d2::PooledConnection<ConnectionManager>;
        type Error = r2d2::Error;

        fn get(&self) -> Result<Self::Connection, Self::Error> {
            self.get()
        }
    }
}
