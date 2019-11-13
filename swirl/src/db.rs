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

/// Object safe version of [`DieselPool`]
pub trait DieselPoolObj {
    /// Object safe version of [`DieselPool::get`]
    ///
    /// This function will heap allocate the connection. This allocation can
    /// be avoided by using [`Self::with_connection`]
    fn get(&self) -> Result<Box<dyn Deref<Target = PgConnection> + '_>, Box<dyn Error>>;

    fn with_connection(
        &self,
        f: &dyn Fn(&PgConnection) -> Result<(), Box<dyn Error>>,
    ) -> Result<(), Box<dyn Error>>;
}

impl<T: DieselPool> DieselPoolObj for T {
    fn get(&self) -> Result<Box<dyn Deref<Target = PgConnection> + '_>, Box<dyn Error>> {
        DieselPool::get(self)
            .map(|v| Box::new(v) as _)
            .map_err(|v| Box::new(v) as _)
    }

    fn with_connection(
        &self,
        f: &dyn Fn(&PgConnection) -> Result<(), Box<dyn Error>>,
    ) -> Result<(), Box<dyn Error>> {
        let conn = DieselPool::get(self)?;
        f(&conn)
    }
}

/// A builder for connection pools
pub trait DieselPoolBuilder {
    /// The concrete connection pool built by this type
    type Pool: DieselPool;

    /// Sets the maximum size of the connection pool.
    fn max_size(self, max_size: u32) -> Self;

    /// Build the pool
    fn build(self, database_url: String) -> Self::Pool;
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

    pub struct R2d2Builder {
        url: String,
        builder: r2d2::Builder<ConnectionManager>,
        connection_count: Option<u32>,
    }

    impl R2d2Builder {
        pub(crate) fn new(url: String, builder: r2d2::Builder<ConnectionManager>) -> Self {
            Self {
                url,
                builder,
                connection_count: None,
            }
        }

        pub(crate) fn connection_count(&mut self, connection_count: u32) {
            self.connection_count = Some(connection_count);
        }

        pub(crate) fn build(self, default_connection_count: u32) -> r2d2::Pool<ConnectionManager> {
            let max_size = self.connection_count.unwrap_or(default_connection_count);
            self.builder
                .max_size(max_size)
                .build_unchecked(ConnectionManager::new(self.url))
        }
    }
}

#[cfg(feature = "r2d2")]
#[doc(hidden)]
pub use self::r2d2_impl::R2d2Builder;
