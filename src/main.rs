#![allow(warnings)]
use actix_cors::Cors;
use actix_web::{
    dev::Path,
    get,
    http::StatusCode,
    web::{self, Data, Json},
    App, HttpResponse, HttpServer, Responder,
};
use chrono::Utc;
use serde::Serialize;
use tokio::time::{interval, sleep, Duration};

mod config;
use config::database_connection;
use config::webauth_utilities::create_webauthn_instance;

mod controllers;
use controllers::*;

mod routes;
use routes::auth::register::{finish_registration, register_start, start_authentication,finish_authentication};

#[get("/")]
async fn index() -> impl Responder {
    "server index route hit"
}

use std::{env, sync::Arc};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Establish the database connection
    let database = database_connection()
        .await
        .expect("Failed to create dbpool");
    println!("Connected to database");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(database.clone()))
            .wrap(
                Cors::default() // Allows all origins
                    .allow_any_origin() // Allows all origins
                    .allow_any_method() // Allows all HTTP methods (GET, POST, etc.)
                    .allow_any_header(), // Allows all headers
            )
            .service(index)
            .service(register_start)
            .service(finish_registration)
            .service(start_authentication)
            .service(finish_authentication)
    })
    .bind(("0.0.0.0", 3001))?
    .run();
    println!("Server running at http://localhost:3001");
    server.await
}
