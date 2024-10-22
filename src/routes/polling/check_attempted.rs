use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{MySql, Pool, Row};
use actix_web::{test, App};

use std::sync::Arc;
use actix_web::web::Data;

#[derive(Deserialize)]
struct AttemptedRequest {
    email: String,
    qid: i32,
}

#[get("/api/question_attempted")]
pub async fn is_question_attempted(
    pool: web::Data<Pool<MySql>>,
    query: web::Query<AttemptedRequest>,
) -> impl Responder {
    println!("/GET question_attempted hit");
    println!("email: {}", query.email.clone());
    println!("qid: {}", query.qid);
    let user_id = query.email.clone();
    // Check if the user has voted on the given question
    let vote_result = sqlx::query(
        r#"
        SELECT COUNT(*) as count FROM votes 
        WHERE user_email = ? AND question_id = ?
        "#,
    )
    .bind(user_id)
    .bind(query.qid)
    .fetch_one(pool.get_ref())
    .await;

    match vote_result {
        Ok(record) => {
            if record.try_get::<i64, _>("count").unwrap_or(0) > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "answered": true
                }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({
                    "answered": false
                }))
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}



