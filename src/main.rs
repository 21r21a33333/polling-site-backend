#![allow(warnings)]
use actix_cors::Cors;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::AUTHORIZATION;
use actix_web::middleware::{from_fn, Next};
use actix_web::web::{self, Data};
use actix_web::{get, App, HttpMessage, HttpResponse, HttpServer, Responder};

use std::{env, sync::Arc};

mod config;
use config::database_connection;
use config::webauth_utilities::create_webauthn_instance;

mod controllers;
use controllers::*;

mod routes;
use jsonwebtoken::errors::Error;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use routes::auth::register::{
    finish_authentication, finish_registration, register_start, start_authentication,
};

use routes::close_poll::close_poll;
use routes::is_question_attempted;
use routes::polling::create_poll::{create_poll, protected_handler};
use routes::polling::get_polls::get_polls;
use routes::polling::get_quiz::get_poll;
use routes::polling::question_scores::get_question_scores;
use routes::polling::vote_handler::crate_vote;
use routes::reset_poll::reset_poll;

// ws
use actix::Actor;
use controllers::websockets::lobby::*;
use controllers::websockets::messages::*;
use controllers::websockets::start_connection::*;
use controllers::websockets::ws::*;
use serde::{Deserialize, Serialize};

// jwt middleware
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Claims {
    sub: String,
    exp: usize,
    // Add other fields as needed
}
async fn jwt_middleware(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    println!("JWT middleware called");
    let headers = req.headers();
    if let Some(auth_header) = headers.get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
                let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
                let validation = Validation::new(Algorithm::HS256);

                match decode::<Claims>(token, &decoding_key, &validation) {
                    Ok(token_data) => {
                        let claims = token_data.claims.clone();
                        req.extensions_mut().insert(claims.clone());
                        req.headers_mut().insert(
                            actix_web::http::header::HeaderName::from_static("user_id"),
                            claims.sub.parse().unwrap(),
                        );
                        println!("user_id in req header: {:?}", req.headers().get("user_id"));
                        return next.call(req).await;
                    }
                    Err(_) => {
                        return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
                    }
                }
            }
        }
    }else{
        return Err(actix_web::error::ErrorUnauthorized("No token provided"));
    }
    next.call(req).await
}

#[get("/")]
async fn index() -> impl Responder {
    "server index route hit"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set the RUST_LOG environment variable to debug
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init();

    // Establish the database connection
    let database = database_connection()
        .await
        .expect("Failed to create dbpool");
    println!("Connected to database");

    //create and spin up a lobby
    let chat_server = Lobby::default().start();

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
            .service(start_connection) //register our route. rename with "as" import or naming conflict
            .app_data(Data::new(chat_server.clone())) //register the lobby
            .service(
                web::scope("")
                .wrap(from_fn(jwt_middleware))
                .service(get_poll) // JWT protected
                .service(get_polls)
                .service(create_poll)
                .service(crate_vote)
                .service(get_question_scores)
                .service(close_poll)
                .service(reset_poll)
                
            )
        })
        .bind(("0.0.0.0", 3001))?
    .run();
    println!("Server running at http://localhost:3001");
    server.await
}
