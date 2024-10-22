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
use shared::{WrappedReport, WrappedReportTs};
use sqlx::types::Json;

use state::AppState;

mod args;
mod config;
mod db;
mod state;

#[actix_web::get("/")]
async fn index() -> impl Responder {
    "Hello from Actix Web! :D"
}

#[actix_web::get("/report")]
async fn get_report(state: AppState, id: actix_web::web::Path<uuid::Uuid>) -> impl Responder {
    // ask the database for the report with this uuid
    let query = sqlx::query_as!(
        WrappedReport,
        r#"SELECT id, recv_time, report as "report: Json<Report>" FROM reports WHERE id = $1"#,
        id.clone()
    )
    .fetch_one(&state.pool)
    .await;

    // ensure we got a report
    let report = match query {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to find a report with the given UUID. (err: {e})");
            return HttpResponse::NotFound()
                .reason("Didn't find the report with the given UUID in the database.")
                .finish();
        }
    };

    // make it into the typescript one
    let ts_report = WrappedReportTs::from(report);

    // jsonify that mf

    HttpResponse::Ok()
        .reason("Successfully pulled report from the database.")
        .json(ts_report)
}

#[actix_web::post("/add_report")]
#[tracing::instrument(skip_all)]
async fn add_report(state: AppState, report: String) -> impl Responder {
    // grab the current time
    let time = chrono::Utc::now();

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

    HttpResponse::Ok().finish()
}

#[actix_web::get("/reports")]
async fn reports(state: AppState) -> impl Responder {
    let query = sqlx::query_as!(
        WrappedReport,
        r#"SELECT id, recv_time, report as "report: Json<Report>" FROM reports"#
    )
    .fetch_all(&state.pool)
    .await;

    let rows = match query {
        Ok(rows) => rows.into_iter().map(WrappedReportTs::from),
        Err(e) => {
            tracing::warn!("Unable to query the database for reports! (err: {e})");
            return HttpResponse::InternalServerError()
                .reason("Failed to query the database.")
                .finish();
        }
    };

    let rows = rows.collect::<Vec<_>>();
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
