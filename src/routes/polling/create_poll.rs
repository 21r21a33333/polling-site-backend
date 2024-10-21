use actix_web::{dev::ServiceRequest, get, web, App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use actix_web::post;
use sqlx::{mysql, query, MySql, Pool, Row};

#[derive(Deserialize)]
struct PollRequest {
    title: String,
    description: Option<String>,
    creator_email: String,
    questions: Vec<QuestionRequest>,
}

#[derive(Deserialize)]
struct QuestionRequest {
    question_text: String,
    options: Vec<String>, // List of option texts
}

#[post("/api/polls")]
pub async fn create_poll(
    pool: web::Data<Pool<MySql>>,
    mut poll_request: web::Json<PollRequest>,
    req: HttpRequest,
) -> impl Responder {
    println!("?POST /api/polls");
    let user_email= req.headers().get("user_id").unwrap().to_str().unwrap();
    println!("creator_email: {}", user_email);
    poll_request.creator_email = user_email.to_string();
    let poll_id = sqlx::query(
        r#"
        INSERT INTO polls (title, description, creator_email)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(&poll_request.title)
    .bind(&poll_request.description)
    .bind(&poll_request.creator_email)
    .execute(pool.get_ref())
    .await
    .unwrap()
    .last_insert_id();

    // Insert questions and options
    for question in &poll_request.questions {
        let question_id: u64 = sqlx::query!(
            r#"
            INSERT INTO questions (poll_id, question_text)
            VALUES (?, ?)
            "#,
            poll_id,
            question.question_text
        )
        .execute(pool.get_ref())
        .await
        .unwrap()
        .last_insert_id();

        for option in &question.options {
            sqlx::query!(
                r#"
                INSERT INTO poll_options (question_id, option_text)
                VALUES (?, ?)
                "#,
                question_id,
                option
            )
            .execute(pool.get_ref())
            .await
            .unwrap();
        }
    }

    HttpResponse::Created().json(serde_json::json!({
        "message": "Poll created successfully",
        "poll_id": poll_id,
    }))
}

// Define a struct for your claims
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String, // Email is stored in 'sub'
    exp: usize,  // Expiration time
}

#[get("/api/protected")]
pub async fn protected_handler(req: HttpRequest) -> Result<impl Responder, Error> {
    // Get the Authorization header
    let auth_header = req.headers().get("Authorization");

    if let Some(header_value) = auth_header {
        // Extract the token from the header
        if let Ok(token) = header_value.to_str() {
            // Remove the "Bearer " prefix
            let token = token.trim_start_matches("Bearer ");

            // Decode the token
            let secret_key = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
            let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
            let validation = Validation::new(Algorithm::HS256); // Use the same algorithm

            match decode::<Claims>(token, &decoding_key, &validation) {
                Ok(token_data) => {
                    let email = token_data.claims.sub; // Extract email from the 'sub' field
                                                       // Return the user email in the response
                    return Ok(HttpResponse::Ok().json(format!("Hello, {}!", email)));
                }
                Err(_) => {
                    return Ok(HttpResponse::Unauthorized().json("Invalid token"));
                }
            }
        }
    }

    Ok(HttpResponse::Unauthorized().json("Authorization header missing or invalid"))
}
