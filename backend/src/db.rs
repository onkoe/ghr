use tokio_postgres::{tls::NoTlsStream, Client, Connection, Error as PostgresError, NoTls, Socket};

use crate::config::config;

const DB_NAME: &str = "ghr_backend";

pub async fn connect() -> Result<(Client, Connection<Socket, NoTlsStream>), PostgresError> {
    let (host, user) = {
        let c = config();
        (c.postgres_host, c.postgres_user)
    };

    tokio_postgres::connect(&format!("host={host} user={user} dbname={DB_NAME}"), NoTls).await
}
