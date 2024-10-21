use std::option;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::{MySql, Pool, Row};

#[derive(Serialize)]
struct OptionScore {
    id: i64,
    option_text: String,
    score: i64,
}

#[derive(Serialize)]
struct QuestionScoresResponse {
    question_id: String,
    options: Vec<OptionScore>,
}

#[get("/api/polls/{poll_id}/questions/{question_id}/scores")]

pub async fn get_question_scores(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (poll_id, question_id) = path.into_inner();
    println!(
        "GET /api/polls/{}/questions/{}/scores",
        poll_id, question_id
    );

    // Fetch the options and their scores for the specified question
    let options = sqlx::query(
        r#"
        SELECT id, option_text, score
        FROM poll_options
        WHERE question_id = ?
        "#,
    )
    .bind(question_id.clone())
    .fetch_all(pool.as_ref())
    .await;

    match options {
        Ok(option_rows) => {
            let option_scores: Vec<OptionScore> = option_rows
                .iter()
                .map(|option| OptionScore {
                    id: option.get("id"),
                    option_text: option.get("option_text"),
                    score: option.get("score"),
                })
                .collect();

            // Construct the response
            let response = QuestionScoresResponse {
                question_id,
                options: option_scores,
            };

            HttpResponse::Ok().json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
