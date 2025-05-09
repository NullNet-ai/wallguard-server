// SSH Gateway
// This module implements a WebSocket server that acts as a bridge between a browser-based UI portal
// and an SSH server. Since browsers cannot open raw SSH (TCP) connections directly, this module serves
// as the intermediary to tunnel SSH communication over WebSocket.

mod session;
mod ssh_message;
mod stop_message;

use crate::app_context::AppContext;
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Payload},
};
use session::Session;

/// @TODO: Remove. Only for development.
const DEV_SSH_ADDR: &str = "192.168.2.46:22";

pub(super) async fn open_ssh_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> actix_web::Result<HttpResponse> {
    let session = Session::new(DEV_SSH_ADDR.parse().unwrap()).await.unwrap();
    actix_web_actors::ws::start(session, &request, body)
}
