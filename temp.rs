use actix_web::{get, web, HttpResponse, Responder};
use sqlx::{MySql, Pool};

#[get("/api/polls/{poll_id}/questions/{question_id}/scores")]
async fn get_question_scores(
    pool: web::Data<Pool<MySql>>,
    web::Path((poll_id, question_id)): web::Path<(i64, i64)>,
) -> impl Responder {
    // Fetch the options and their scores for the specified question
    let options = sqlx::query!(
        r#"
        SELECT id, text, score
        FROM poll_options
        WHERE question_id = ?
        "#,
        question_id
    )
    .fetch_all(pool.as_ref())
    .await;

    match options {
        Ok(option_rows) => {
            // Map the database rows to the OptionScore struct
            let option_scores: Vec<OptionScore> = option_rows
                .iter()
                .map(|option| OptionScore {
                    id: option.id,
                    text: option.text.clone(),
                    score: option.score,
                })
                .collect();

            // Construct the response
            let response = QuestionScoresResponse {
                question_id,
                options: option_scores,
            };

            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to fetch scores."),
    }
}
