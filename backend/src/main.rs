//! # `backend`
//!
//! This is the backend crate for the GHR project's website.
//!
//! ## Setup
//!
//! Make sure PostgreSQL is installed, then make a user + table for it.
//!
//! If you're getting some kind of authentication error when you start the
//! program, make sure local users have `md5` (kinda secure) or `trust`
//! (insecure, for testing) instead of `ident`. Make sure to reload the
//! service's config after changing these options.
//!
//! To make the user, you use `sudo -u postgres psql`, then
//! `CREATE USER (yourname) WITH PASSWORD '(YOURPASS)';`. I personally use
//! `farts` as my password - hope that helps.
//!
//! ## Usage
//!
//! First, make sure Postgres is up and running: `sudo systemctl start
//! postgres`. You can now run this backend from a binary or using
//! `cargo run -- (your args)`. Enjoy!

use crate::{args::Arguments, state::State};

use actix_web::{web::Data, App, HttpResponse, HttpServer, Responder};
use clap::Parser as _;
use libghr::report::Report;
use sqlx::types::{time::OffsetDateTime, Json};

use state::AppState;

mod args;
mod config;
mod db;
mod state;

#[derive(sqlx::FromRow)]
struct ReportRow {
    recv_time: OffsetDateTime,
    report: Json<Report>,
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
    "Hello from Actix Web! :D"
}

#[actix_web::post("/add_report")]
#[tracing::instrument(skip_all)]
async fn add_report(state: AppState, report: String) -> impl Responder {
    // grab the current time
    let time = OffsetDateTime::now_utc();

    // try making an object from the report
    let parsed_report: Report = match serde_json::from_str(&report) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Failed to serialize user-passed `Report`. (err: {e})");
            return HttpResponse::InternalServerError()
                .reason("Failed to serialize given report to `serde_json::Value`.")
                .finish();
        }
    };

    let query = sqlx::query!(
        r#"
        INSERT INTO reports (recv_time, report)
        VALUES ($1, $2)
        "#,
        time,
        Json(parsed_report) as _
    )
    .execute(&state.pool)
    .await;

    if let Err(e) = query {
        tracing::warn!("Unable to query the database for reports! (err: {e})");
        return HttpResponse::InternalServerError()
            .reason("Failed to query the database.")
            .finish();
    }

    HttpResponse::Ok().json("TODO")
}

#[actix_web::get("/reports")]
async fn reports(state: AppState) -> impl Responder {
    let query = sqlx::query_as!(
        ReportRow,
        r#"SELECT recv_time, report as "report: Json<Report>" FROM reports"#
    )
    .fetch_all(&state.pool)
    .await;

    let rows = match query {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("Unable to query the database for reports! (err: {e})");
            return HttpResponse::InternalServerError()
                .reason("Failed to query the database.")
                .finish();
        }
    };

    let rows = rows.into_iter().map(|row| row.report).collect::<Vec<_>>();
    HttpResponse::Ok().json(rows)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create our config
    let args = Arguments::parse();
    crate::config::init(args);

    // start logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    tracing::info!("The backend is now starting...");

    // make connection to the database
    tracing::info!("Connecting to database...");
    let pool = db::pool().await.map_err(std::io::Error::other)?;

    // create the app's state
    let state = State { pool };

    tracing::info!("Connected! The server is now running...");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .service(add_report)
            .service(index)
            .service(reports)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}