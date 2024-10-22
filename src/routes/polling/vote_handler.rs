use actix::Addr;
use actix_web::{
    post,
    web::{self, Data},
    HttpRequest, HttpResponse, Responder,
};
use sqlx::{MySql, Pool, Row};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation}; // Add dependencies for JWT decoding
use crate::{Lobby, NotifyPollId};

// Define your claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,       // user email
    option_id: String, // option ID
    exp: usize,        // expiration time
}


#[post("/api/polls/{poll_id}/vote")]
pub async fn crate_vote(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<(String)>,
    req: HttpRequest,
    srv: Data<Addr<Lobby>>,
) -> impl Responder {
    let poll_id: i64 = path.into_inner().parse().unwrap();
    println!("POST /api/polls/{}/vote", poll_id);




    // Get the token from the authorization header
    let token = match req.headers().get("Authentication") {
        Some(header_value) => header_value.to_str().unwrap_or("").to_string(),
        None => return HttpResponse::BadRequest().json("Missing token"),
    };

    // Remove "Bearer " from the token if present
    let token = token.strip_prefix("Bearer ").unwrap_or(&token);

    // Decode the token to get claims
    let secret_key = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = Validation::default();
    let decoded_token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &validation,
    );

    let my_claims = match decoded_token {
        Ok(c) => c.claims,
        Err(_) => return HttpResponse::Unauthorized().json("Invalid token"),
    };

    // Now you can use my_claims.sub and my_claims.option_id in your logic
    let user_id = my_claims.sub;
    let option_id = my_claims.option_id;

    // Check if the poll exists and is open
    let poll_exists = sqlx::query!(
        r#"
        SELECT closed FROM polls WHERE id = ?
        "#,
        poll_id
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    if poll_exists.closed == Some(1) {
        return HttpResponse::BadRequest().json("Poll is closed.");
    }

    // Check if the question with that option exists and check if user already voted
    let question_exists = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM poll_options
        WHERE id = ? AND question_id IN (
            SELECT id FROM questions WHERE poll_id = ?
        )
        "#,
        option_id,
        poll_id
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    let count: i64 = question_exists.count;

    if count == 0 {
        return HttpResponse::BadRequest().json("Invalid option for this poll.");
    }

    let already_voted = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM votes
        WHERE question_id = (
            SELECT question_id FROM poll_options WHERE id = ?
        ) AND user_email = ?
        "#,
        option_id,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    let count: i64 = already_voted.count;

    if count > 0 {
        return HttpResponse::BadRequest().json("User has already voted for this question.");
    }

    // Check if the user has already voted for this option
    let already_voted = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM votes 
        WHERE option_id = ? AND user_email = ?
        "#,
    )
    .bind(&option_id)
    .bind(user_id.clone())
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    let count: i64 = already_voted.get("count");

    if count > 0 {
        return HttpResponse::BadRequest().json("User has already voted for this option.");
    }

    // Insert the vote into the database
    let _ = sqlx::query(
        r#"
        INSERT INTO votes (question_id, option_id, user_email)
        VALUES (
            (SELECT question_id FROM poll_options WHERE id = ?),
            ?,
            ?
        )
        "#,
    )
    .bind(&option_id)
    .bind(&option_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await
    .unwrap();

    // Update the score in the poll_options table
    let _ = sqlx::query(
        r#"
        UPDATE poll_options
        SET score = score + 1
        WHERE id = ?
        "#,
    )
    .bind(&option_id)
    .execute(pool.get_ref())
    .await
    .unwrap();

    // Notify the lobby of the vote
    srv.send(NotifyPollId {
        poll_id: poll_id.clone(),
    })
    .await
    .map_err(|e| {
        eprintln!("Error sending message to lobby: {:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    });

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Vote created"
    }))
}
