use actix_web::{post, web, HttpResponse, Responder};
use reqwest::Client;
use sqlx::MySqlPool;

use super::StartAuthenticationRequest;

#[post("/start_verification")]
pub async fn start_verification(
    pool: web::Data<MySqlPool>, // Your MySQL connection pool
    req_body: web::Json<StartAuthenticationRequest>,
) -> impl Responder {
    println!("POST /login/start_verification");

    // Create a new HTTP client
    let client = Client::new();
    let auth_url = "http://0.0.0.0:3001/login/start".to_string(); // Set your base URL

    // Convert req_body to JSON
    let req_json = serde_json::to_value(&req_body.into_inner()).unwrap();

    // Send a request to the start_authentication route
    let response = client
        .post(&auth_url)
        .json(&req_json)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "Failed to read response body".to_string());

            // Return the response status and body from start_authentication
            HttpResponse::build(actix_web::http::StatusCode::from_u16(status.as_u16()).unwrap()).body(body)
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}