use clap::Parser;

#[derive(Parser, Debug)]
pub(crate) struct Arguments {
    /// The user that hosts the Postgres database. (typically `postgres`)
    #[arg(short = 'u', long, default_value_t = String::from("postgres"))]
    pub postgres_user: String,
    /// (I think) the IP of the computer hosting the server. (TODO)
    #[arg(short = 'i', long, default_value_t = String::from("localhost"))]
    pub postgres_host: String,
}
