use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};

#[derive(Serialize)]
struct PollListResponse {
    id: i64,
    title: String,
    description: Option<String>,
    creator_email: String,
    created_at: String,
    closed: bool,
}

#[derive(Deserialize)]
struct PollStatusQuery {
    closed: Option<bool>, // Optional query param, defaults if not provided
    creator: Option<String>,
}

#[get("/api/polls")]
pub async fn get_polls(
    pool: web::Data<Pool<MySql>>,
    query: web::Query<PollStatusQuery>,
) -> impl Responder {
    let closed_value = query.closed.clone().unwrap_or(false);
    println!("/GET polls?status={}", closed_value);
    // Fetch polls based on the closed value
    // if crateor is provided, fetch polls created by the creator
    if let Some(creator) = &query.creator {
        let polls_result = sqlx::query!(
            r#"
            SELECT id, title, description, creator_email, created_at, closed
            FROM polls
            WHERE closed = ? AND creator_email = ?
            order by created_at desc
            "#,
            closed_value,
            creator
        )
        .fetch_all(pool.as_ref())
        .await;

        match polls_result {
            Ok(polls) => {
                let poll_list: Vec<PollListResponse> = polls
                    .iter()
                    .map(|poll| PollListResponse {
                        id: poll.id as i64,
                        title: poll.title.clone(),
                        description: poll.description.clone(),
                        creator_email: poll.creator_email.clone(),
                        created_at: poll
                            .created_at
                            .map_or_else(|| "".to_string(), |dt| dt.to_string()),
                        closed: poll.closed.unwrap_or(0) != 0,
                    })
                    .collect();

                return HttpResponse::Ok().json(poll_list);
            }
            Err(_) => return HttpResponse::InternalServerError().json("Failed to fetch polls."),
        }
    }
    let polls_result = sqlx::query!(
        r#"
        SELECT id, title, description, creator_email, created_at, closed
        FROM polls
        WHERE closed = ?
        order by created_at desc
        "#,
        closed_value
    )
    .fetch_all(pool.as_ref())
    .await;

    match polls_result {
        Ok(polls) => {
            let poll_list: Vec<PollListResponse> = polls
                .iter()
                .map(|poll| PollListResponse {
                    id: poll.id as i64,
                    title: poll.title.clone(),
                    description: poll.description.clone(),
                    creator_email: poll.creator_email.clone(),
                    created_at: poll
                        .created_at
                        .map_or_else(|| "".to_string(), |dt| dt.to_string()),
                    closed: poll.closed.unwrap_or(0) != 0,
                })
                .collect();

            HttpResponse::Ok().json(poll_list)
        }
        Err(_) => HttpResponse::InternalServerError().json("Failed to fetch polls."),
    }
}
