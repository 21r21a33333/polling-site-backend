use actix::Addr;
use actix_web::{
    post, web::{self, Data}, HttpMessage, HttpRequest, HttpResponse, Responder
};
use serde::Deserialize;
use sqlx::{MySql, Pool, Row};

use crate::{Lobby, NotifyPollId};

#[derive(Deserialize)]
struct ClosePollRequest {
    email: String, // Creator's email to verify ownership
}

#[post("/api/polls/{poll_id}/close")]
pub async fn close_poll(
    pool: web::Data<Pool<MySql>>,
    path: web::Path<(String)>,
    req: web::Json<ClosePollRequest>,
    srv: Data<Addr<Lobby>>,
    request:HttpRequest
) -> impl Responder {
    let poll_id = path.into_inner();
    println!("/POST polls/{}/close", poll_id);

    // Check if the poll exists and if the creator matches the given email
    let poll = sqlx::query(
        r#"
        SELECT creator_email, closed
        FROM polls
        WHERE id = ? AND closed = FALSE
        "#,
    )
    .bind(poll_id.clone())
    .fetch_one(pool.as_ref())
    .await;

    match poll {
        Ok(p) => {
            let mut poll_creator=p.get::<String, _>("creator_email");
            let req_user=request.headers().get("user_id").unwrap().to_str().unwrap();

            if poll_creator!=req_user{
                return HttpResponse::Unauthorized().json("You are not authorized to close this poll.");
            }
            // Check if the requester is the creator
            if  poll_creator == req.email {
                // Update the poll to close it
                let update_result = sqlx::query(
                    r#"
                    UPDATE polls
                    SET closed = TRUE
                    WHERE id = ?
                    "#,
                )
                .bind(poll_id.clone())
                .execute(pool.as_ref())
                .await;

                match update_result {
                    Ok(_) => {
                        srv.send(NotifyPollId {
                            poll_id: poll_id.parse::<i64>().unwrap(),
                        })
                        .await
                        .map_err(|e| {
                            eprintln!("Error sending message to lobby: {:?}", e);
                            actix_web::error::ErrorInternalServerError(e)
                        });
                        HttpResponse::Ok().json("Poll closed successfully.")
                    }
                    Err(_) => HttpResponse::InternalServerError().json("Failed to close the poll."),
                }
            } else {
                HttpResponse::Unauthorized().json("You are not authorized to close this poll.")
            }
        }
        Err(_) => HttpResponse::NotFound().json("Poll not found or already closed."),
    }
}
