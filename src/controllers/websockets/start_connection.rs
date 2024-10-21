use crate::lobby::Lobby;
use crate::messages::NotifyPollId;
use crate::{messages::GetOrCreateGroup, ws::WsConn};
use actix::Addr;
use actix_web::{
    get,
    web::{Data, Path, Payload},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;

#[get("ws/{poll_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    poll_id: Path<i64>, // poll_id as i64
    srv: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    println!("start_connection");
    // Get or create a group for the poll_id
    let group_id = {
        let mut lobby = srv.get_ref().clone();
        match lobby
            .send(GetOrCreateGroup {
                poll_id: poll_id.into_inner(),
            })
            .await
        {
            Ok(Ok(group_id)) => group_id,
            Ok(Err(e)) => {
                return Err(actix_web::error::ErrorInternalServerError(format!(
                    "Error: {:?}",
                    e
                )))
            }
            Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
        } // Assuming an appropriate message
    };

    // Create the WebSocket connection for the group
    let ws = WsConn::new(group_id, srv.get_ref().clone());
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

#[get("ws/third-party/{poll_id}")]
pub async fn notify_poll_id(
    poll_id: Path<i64>, // poll_id as i64
    srv: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    // Send a message to the Lobby actor with the poll_id
    srv.send(NotifyPollId {
        poll_id: poll_id.clone(),
    })
    .await
    .map_err(|e| {
        eprintln!("Error sending message to lobby: {:?}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    Ok(HttpResponse::Ok().body(format!("Notified lobby for poll_id: {}", poll_id)))
}
