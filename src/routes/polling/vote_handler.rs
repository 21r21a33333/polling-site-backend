use actix_web::{post, web, HttpResponse, Responder};
use sqlx::{MySql, Pool, Row};

use serde::Deserialize;

#[derive(Deserialize)]
struct VoteRequest {
    email: String,
    option_id: String,
}


#[post("/api/polls/{poll_id}/vote")]
pub async fn crate_vote(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<(String)>,
    vote_request: web::Json<VoteRequest>,
) -> impl Responder {
    let (poll_id) = path.into_inner();
    println!("POST /api/polls/{}/vote", poll_id);
    let user_id=&vote_request.email;

    // check if hte poll exists and is open
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

    // check if the user has already voted for this option
    let already_voted = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM votes 
        WHERE option_id = ? AND user_email = ?
        "#
    )
    .bind(&vote_request.option_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    let count: i64 = already_voted.get("count");

    if count > 0 {
        return HttpResponse::BadRequest().json("User has already voted for this option.");
    }

    let _=sqlx::query(
        r#"
        INSERT INTO votes (question_id, option_id, user_email)
        VALUES (
            (SELECT question_id FROM poll_options WHERE id = ?),
            ?,
            ?
        )
        "#
    ).bind(&vote_request.option_id)
    .bind(&vote_request.option_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await
    .unwrap();


    //update the score in the poll_options table
    let _=sqlx::query(
        r#"
        UPDATE poll_options
        SET score = score + 1
        WHERE id = ?
        "#
    ).bind(&vote_request.option_id)
    .execute(pool.get_ref())
    .await
    .unwrap();

    HttpResponse::Ok().json(serde_json::json!({
        "message": "vote created"
    }))
}