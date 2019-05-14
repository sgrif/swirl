use diesel::PgConnection;
use std::error::Error;
use std::ops::Deref;

pub type DieselPooledConn<'a, T> = <T as BorrowedConnection<'a>>::Connection;

/// A trait to work around associated type constructors
///
/// This will eventually change to `type Connection<'a>` on [`DieselPool`]
pub trait BorrowedConnection<'a> {
    /// The smart pointer returned by this connection pool.
    type Connection: Deref<Target = PgConnection>;
}

/// A connection pool for Diesel database connections
///
/// If you don't care about the details of connection pooling, or want to use
/// the r2d2 crate, you can enable the r2d2 feature on this crate and never
/// be concerned with this trait. If you want to use your own connection pool,
/// you can implement this trait manually.
pub trait DieselPool: Clone + Send + for<'a> BorrowedConnection<'a> {
    /// The error type returned when a connection could not be retreived from
    /// the pool.
    type Error: Error + Send + Sync + 'static;

    /// Attempt to get a database connection from the pool. Errors if a
    /// connection could not be retrieved from the pool.
    ///
    /// The exact details of why an error would be returned will depend on
    /// the pool, but a reasonable implementation will return an error if:
    ///
    /// - A timeout was reached
    /// - An error occurred establishing a new connection
    fn get(&self) -> Result<DieselPooledConn<'_, Self>, Self::Error>;
}

#[cfg(feature = "r2d2")]
mod r2d2_impl {
    use super::*;
    use diesel::r2d2;

    type ConnectionManager = r2d2::ConnectionManager<PgConnection>;

    impl<'a> BorrowedConnection<'a> for r2d2::Pool<ConnectionManager> {
        type Connection = r2d2::PooledConnection<ConnectionManager>;
    }

    impl DieselPool for r2d2::Pool<ConnectionManager> {
        type Error = r2d2::PoolError;

        fn get<'a>(&'a self) -> Result<DieselPooledConn<'a, Self>, Self::Error> {
            self.get()
        }
    }
}
