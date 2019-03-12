use diesel::r2d2;
use diesel::prelude::*;

pub type DieselPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub fn pool_builder() -> r2d2::Builder<r2d2::ConnectionManager<PgConnection>> {
    r2d2::Pool::builder()
        .max_size(4)
        .min_idle(Some(0))
        .connection_customizer(Box::new(SetStatementTimeout(1000)))
}

#[derive(Debug, Clone, Copy)]
struct SetStatementTimeout(u64);

impl r2d2::CustomizeConnection<PgConnection, r2d2::Error> for SetStatementTimeout {
    fn on_acquire(&self, conn: &mut PgConnection) -> Result<(), r2d2::Error> {
        use diesel::sql_query;

        sql_query(format!("SET statement_timeout = {}", self.0))
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;
        Ok(())
    }
}
