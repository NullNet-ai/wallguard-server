use crate::{
    grpc_server::server::WallGuardImpl,
    parser::msg_parser::parse_message,
    proto::wallguard::{CommonResponse, Packets},
};
use nullnet_liberror::Error;
use tonic::{Request, Response};

impl WallGuardImpl {
    pub(crate) async fn handle_packets_impl(
        &self,
        request: Request<Packets>,
    ) -> Result<Response<CommonResponse>, Error> {
        let packets = request.into_inner();
        let (jwt_token, token_info) = Self::authenticate(&packets.token)?;

        log::info!("Received {} packets.", packets.packets.len());
        let parsed_message = parse_message(packets, &token_info, &self.ip_info_tx);
        log::info!("Parsed {} connections.", parsed_message.records.len());

        if parsed_message.records.is_empty() {
            return Ok(Response::new(CommonResponse {
                message: "No valid connections in the message (skipping insertion to datastore)"
                    .to_string(),
            }));
        }

        let _ = self
            .context
            .datastore
            .connections_insert(&jwt_token, parsed_message)
            .await?;

        Ok(Response::new(CommonResponse {
            message: String::from("Connections successfully inserted"),
        }))
    }
}
