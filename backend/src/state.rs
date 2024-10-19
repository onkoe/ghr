use actix_web::web::Data;
use sqlx::{Pool, Postgres};

/// The server's state (for the database, maybe with some caching).
#[derive(Clone)]
pub(super) struct State {
    /// The 'pool' of database connections. See `deadpool-postgres` crate for
    /// more info.
    pub pool: Pool<Postgres>,
}

pub(super) type AppState = Data<State>;
