use diesel::PgConnection;
use std::error::Error;
use std::ops::Deref;

/// A connection pool for Diesel database connections
///
/// If you don't care about the details of connection pooling, or want to use
/// the r2d2 crate, you can enable the r2d2 feature on this crate and never
/// be concerned with this trait. If you want to use your own connection pool,
/// you can implement this trait manually.
pub trait DieselPool<'a>: Clone + Send {
    /// The smart pointer returned by this connection pool.
    type Connection: Deref<Target = PgConnection>;

    /// The error type returned when a connection could not be retreived from
    /// the pool.
    type Error: Error + 'static;

    /// Attempt to get a database connection from the pool. Errors if a
    /// connection could not be retrieved from the pool.
    ///
    /// The exact details of why an error would be returned will depend on
    /// the pool, but a reasonable implementation will return an error if:
    ///
    /// - A timeout was reached
    /// - An error occurred establishing a new connection
    fn get(&'a self) -> Result<Self::Connection, Self::Error>;
}

/// A helper trait for `for<'a> DieselPool<'a>`
pub trait DieselPoolOwned: for<'a> DieselPool<'a> {}
impl<T> DieselPoolOwned for T
where
    for<'a> T: DieselPool<'a>,
{}

#[cfg(feature = "r2d2")]
mod r2d2_impl {
    use super::*;
    use diesel::r2d2;

    type ConnectionManager = r2d2::ConnectionManager<PgConnection>;

    impl<'a> DieselPool<'a> for r2d2::Pool<ConnectionManager> {
        type Connection = r2d2::PooledConnection<ConnectionManager>;
        type Error = r2d2::PoolError;

        fn get(&'a self) -> Result<Self::Connection, Self::Error> {
            self.get()
        }
    }
}
