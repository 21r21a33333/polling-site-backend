use actix::Addr;
use actix_web::{
    post,
    web::{self, Data},
    HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::{MySql, Pool};

use crate::{Lobby, NotifyPollId};

#[derive(Deserialize)]
struct RestartPollRequest {
    email: String, // Creator's email to verify ownership
}

#[post("/api/polls/{poll_id}/reset")]
pub async fn reset_poll(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<(String)>,
    req: web::Json<RestartPollRequest>,
    srv: Data<Addr<Lobby>>,
    request: HttpRequest,
) -> impl Responder {
    let poll_id = path.into_inner();
    let header_user_id = request.headers().get("user_id").unwrap().to_str().unwrap();
    let poll = sqlx::query!(
        r#"
        SELECT creator_email, closed
        FROM polls
        WHERE id = ? AND closed = FALSE
        "#,
        poll_id
    )
    .fetch_one(pool.as_ref())
    .await;
    match poll {
        Ok(p) => {
            if p.creator_email != header_user_id {
                return HttpResponse::Unauthorized()
                    .json("authorization credentials doesnot match.");
            }
            // Check if the requester is the creator
            if p.creator_email == req.email {
                // Reset the votes for all options under the poll's questions
                let reset_result = sqlx::query!(
                    r#"
                    UPDATE poll_options
                    SET score = 0
                    WHERE question_id IN (
                        SELECT id FROM questions WHERE poll_id = ?
                    )
                    "#,
                    poll_id
                )
                .execute(pool.as_ref())
                .await;
                let delete_votes_result = sqlx::query!(
                    r#"
                        DELETE FROM votes
                        WHERE option_id IN (
                            SELECT id FROM poll_options WHERE question_id IN (
                                SELECT id FROM questions WHERE poll_id = ?
                            )
                        )
                        "#,
                    poll_id
                )
                .execute(pool.as_ref())
                .await;

                match delete_votes_result {
                    Ok(_) => {
                        srv.send(NotifyPollId {
                            poll_id: poll_id.parse::<i64>().unwrap(),
                        })
                        .await
                        .map_err(|e| {
                            eprintln!("Error sending message to lobby: {:?}", e);
                            actix_web::error::ErrorInternalServerError(e)
                        });
                        HttpResponse::Ok().json("Poll reset successfully.")
                    }
                    Err(_) => HttpResponse::InternalServerError().json("Failed to reset the poll."),
                }
            } else {
                HttpResponse::Unauthorized().json("You are not authorized to reset this poll.")
            }
        }
        Err(_) => HttpResponse::NotFound().json("Poll not found or already closed."),
    }
}
