use actix::Addr;
use actix_web::{
    post, web::{self, Data}, HttpRequest, HttpResponse, Responder
};
use sqlx::{MySql, Pool, Row};

use serde::Deserialize;

use crate::{Lobby, NotifyPollId};

#[derive(Deserialize)]
struct VoteRequest {
    email: String,
    option_id: String,
}

#[post("/api/polls/{poll_id}/vote")]
pub async fn crate_vote(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<(String)>,
    req: HttpRequest,
    vote_request: web::Json<VoteRequest>,
    srv: Data<Addr<Lobby>>,
) -> impl Responder {
    let poll_id: i64 = path.into_inner().parse().unwrap();
    println!("POST /api/polls/{}/vote", poll_id);

    let user_id = &vote_request.email;
    let header_user_id = req.headers().get("user_id").unwrap().to_str().unwrap();

    if user_id != &header_user_id {
        return HttpResponse::BadRequest().json("Authorization error");
    }
    let user_id = header_user_id.to_string();
    
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

    //chek if the question with that option exists and check if user already voted
    let question_exists = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM poll_options
        WHERE id = ? AND question_id IN (
            SELECT id FROM questions WHERE poll_id = ?
        )
        "#,
        vote_request.option_id,
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
        vote_request.option_id,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    let count: i64 = already_voted.count;

    if count > 0 {
        return HttpResponse::BadRequest().json("User has already voted for this question.");
    }

    // check if the user has already voted for this option
    let already_voted = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM votes 
        WHERE option_id = ? AND user_email = ?
        "#,
    )
    .bind(&vote_request.option_id)
    .bind(user_id.clone())
    .fetch_one(pool.get_ref())
    .await
    .unwrap();

    let count: i64 = already_voted.get("count");

    if count > 0 {
        return HttpResponse::BadRequest().json("User has already voted for this option.");
    }

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
    .bind(&vote_request.option_id)
    .bind(&vote_request.option_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await
    .unwrap();

    //update the score in the poll_options table
    let _ = sqlx::query(
        r#"
        UPDATE poll_options
        SET score = score + 1
        WHERE id = ?
        "#,
    )
    .bind(&vote_request.option_id)
    .execute(pool.get_ref())
    .await
    .unwrap();

    srv.send(NotifyPollId {
        poll_id: poll_id.clone(),
    })
    .await
    .map_err(|e| {
        eprintln!("Error sending message to lobby: {:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    });

    HttpResponse::Ok().json(serde_json::json!({
        "message": "vote created"
    }))
}
