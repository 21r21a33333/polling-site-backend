use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, Row};
#[derive(Serialize, Deserialize)]
struct PollOption {
    id: i64,
    option_text: String,
    score: i64, // Score for the option
}

#[derive(Serialize, Deserialize)]
struct Question {
    id: i64,
    question_text: String,
    options: Vec<PollOption>,
}

#[derive(Serialize, Deserialize)]
struct PollResponse {
    id: i64,
    title: String,
    description: Option<String>,
    creator_email: String, // Changed from user ID to email
    created_at: String,    // You may want to use a DateTime type
    questions: Vec<Question>,
    closed: bool,
}

#[get("/api/polls/{poll_id}")]
pub async fn get_poll(pool: web::Data<Pool<MySql>>, path: web::Path<(String)>) -> impl Responder {
    let (poll_id) = path.into_inner();
    println!("GET /api/polls/{poll_id}");

    let poll_details = sqlx::query!(
        r#"
        SELECT id, title, description, creator_email, created_at , closed
        FROM polls
        WHERE id = ?
        "#,
        poll_id
    )
    .fetch_one(pool.as_ref())
    .await;

    match poll_details {
        Ok(poll) => {
            let questions = sqlx::query(
                r#"
                SELECT id, question_text
                FROM questions
                WHERE poll_id = ?
                "#,
            )
            .bind(poll_id)
            .fetch_all(pool.as_ref())
            .await;

            match questions {
                Ok(question_rows) => {
                    let mut question_vec = Vec::new();
                    for question in question_rows {
                        let options = sqlx::query(
                            r#"
                            SELECT id, option_text, score
                            FROM poll_options
                            WHERE question_id = ?
                            "#,
                        )
                        .bind(question.get::<i64, _>("id"))
                        .fetch_all(pool.as_ref())
                        .await;
                        let options_result = options.map(|option_rows| {
                            option_rows
                                .iter()
                                .map(|option| PollOption {
                                    id: option.get("id"),
                                    option_text: option.get::<String, _>("option_text").clone(),
                                    score: option.get("score"),
                                })
                                .collect::<Vec<PollOption>>()
                        });

                        if let Ok(option_vec) = options_result {
                            question_vec.push(Question {
                                id: question.get("id"),
                                question_text: question.get::<String, _>("question_text").clone(),
                                options: option_vec,
                            });
                        }
                    }

                    let poll_response = PollResponse {
                        id: poll.id as i64,
                        title: poll.title,
                        description: poll.description,
                        creator_email: poll.creator_email,
                        created_at: poll
                            .created_at
                            .map_or_else(|| "".to_string(), |dt| dt.to_string()),
                        questions: question_vec,
                        closed: poll.closed.unwrap_or(0) != 0,
                    };

                    return HttpResponse::Ok().json(poll_response);
                }
                Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
            }
        }
        Err(err) => HttpResponse::NotFound().json(err.to_string()),
    }
}
