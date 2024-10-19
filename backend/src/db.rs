use actix_web::HttpResponse;
use anyhow::anyhow;
use sqlx::{pool::PoolConnection, postgres::PgPoolOptions, Pool, Postgres};

use crate::{config::config, state::AppState};

const DB_NAME: &str = "ghr_backend";

/// creates a 'pool' of database connections.
#[tracing::instrument]
pub(super) async fn pool() -> anyhow::Result<Pool<Postgres>> {
    // grab the configuration for the entire backend, including cli args
    let c = config();
    let username = c.postgres_user;
    let host = c.postgres_host;

    // create a link that opens the connection
    let link = format!("postgres://{username}@{host}/{DB_NAME}");

    // define configuration for how to connect to the postgres db
    PgPoolOptions::new()
        .max_connections(32)
        .connect(&link)
        .await
        .map_err(|e| anyhow!(e))
}

/// gets a connection to the database from the pool.
#[tracing::instrument(skip_all)]
pub(super) async fn get(state: &AppState) -> Result<PoolConnection<Postgres>, HttpResponse> {
    match state.pool.acquire().await {
        Ok(conn) => Ok(conn),
        Err(e) => {
            tracing::error!("Couldn't connect to database! (err: {e})");
            Err(HttpResponse::InternalServerError()
                .reason("Failed to connect to reports database.")
                .finish())
        }
    }
}
