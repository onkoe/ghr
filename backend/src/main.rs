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

use std::time::Instant;

use actix_web::{get, post, web::Json, App, HttpResponse, HttpServer, Responder};
use args::Arguments;
use clap::Parser as _;
use libghr::report::Report;

use crate::config::Config;

mod args;
mod config;
mod db;

#[get("/")]
async fn index() -> impl Responder {
    "Hello from Actix Web! :D"
}

#[post("/add_report")]
async fn add_report(report: Json<Report>) -> impl Responder {
    let _recv_time = Instant::now();
    let _db = [report.0];

    HttpResponse::Ok().json("TODO")
}

#[get("/reports")]
async fn reports() -> impl Responder {
    let report = Report::new().await.expect("report should magically work");
    HttpResponse::Ok().json([report])
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create our config
    let args = Arguments::parse();
    crate::config::init(args);

    // start logging
    tracing_subscriber::fmt().init();
    tracing::info!("The backend is now starting...");

    // make connection to the database
    tracing::info!("Connecting to database...");
    let (client, connection) = db::connect().await.map_err(std::io::Error::other)?;

    tracing::info!("Connected! The server is now running...");
    HttpServer::new(|| {
        App::new()
            .service(add_report)
            .service(index)
            .service(reports)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
