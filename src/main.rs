#![allow(warnings)]
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{get, App, HttpServer, Responder};

mod config;
use config::database_connection;
use config::webauth_utilities::create_webauthn_instance;

mod controllers;
use controllers::*;

mod routes;
use routes::auth::register::{
    finish_authentication, finish_registration, register_start, start_authentication,
};

use routes::close_poll::close_poll;
use routes::is_question_attempted;
use routes::polling::create_poll::{create_poll, protected_handler};
use routes::polling::get_quiz::get_poll;
use routes::polling::vote_handler::crate_vote;
use routes::polling::question_scores::get_question_scores;
use routes::polling::get_polls::get_polls;
use routes::reset_poll::reset_poll;

#[get("/")]
async fn index() -> impl Responder {
    "server index route hit"
}

// create a sample protected route
#[get("/api/protected")]
pub async fn protected_route() -> impl Responder {
    "Protected route hit"
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
            .service(create_poll)
            .service(protected_handler)
            .service(get_poll)
            .service(crate_vote)
            .service(get_question_scores)
            .service(get_polls)
            .service(close_poll)
            .service(reset_poll)
            .service(is_question_attempted)
    })
    .bind(("0.0.0.0", 3001))?
    .run();
    println!("Server running at http://localhost:3001");
    server.await
}
