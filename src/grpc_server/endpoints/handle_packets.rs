use crate::datastore::DatastoreWrapperExperimental;
use crate::proto::wallguard::wall_guard_server;
use crate::{
    grpc_server::server::WallGuardImpl,
    parser::{msg_parser::parse_message, parsed_message::ParsedRecord},
    proto::wallguard::{CommonResponse, Packets},
};
use nullnet_libdatastore::store::{self};
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

impl WallGuardImpl {
    pub(crate) async fn handle_packets_impl(
        &self,
        request: Request<Streaming<Packets>>,
    ) -> Result<
        Response<<WallGuardImpl as wall_guard_server::WallGuard>::HandlePacketsStream>,
        Status,
    > {
        let mut stream = request.into_inner();

        let ip_info_tx = self.ip_info_tx.clone();
        let ds = self.context.datastore.clone();
        let ds_exp = self.context.datastore_exp.clone();

        let output = async_stream::try_stream! {
            while let Some(packets_res) = stream.next().await {
                let Ok(packets) = packets_res else {
                    continue;
                };

                let (jwt_token, token_info) = Self::authenticate(&packets.token).
                map_err(|e| Status::internal(format!("{e:?}")))?;

                log::info!("Received {} packets.", packets.packets.len());
                let parsed_message = parse_message(packets, &token_info, &ip_info_tx);
                log::info!("Parsed {} connections.", parsed_message.records.len());

                if parsed_message.records.is_empty() {
                    yield CommonResponse {
                        message:
                            "No valid connections in the message (skipping insertion to datastore)"
                                .to_string(),
                    };
                }

                Self::experimental_create_connections(ds_exp.clone(), &parsed_message.records)
                    .await;

                let _ = ds
                    .connections_insert(&jwt_token, parsed_message)
                    .await.map_err(|e| Status::internal(format!("{e:?}")))?;

                yield CommonResponse {
                    message: String::from("Connections inserted successfully"),
                };
            }
        };

        Ok(Response::new(
            Box::pin(output)
                as <WallGuardImpl as wall_guard_server::WallGuard>::HandlePacketsStream,
        ))
    }

    /**
     * This code is to be removed
     */
    async fn experimental_create_connections(
        ds_exp: Option<DatastoreWrapperExperimental>,
        connections: &[ParsedRecord],
    ) {
        if ds_exp.is_none() || connections.is_empty() {
            return;
        }

        let connection = &connections[0];

        let conn = store::Connections {
            tombstone: None,
            status: None,
            previous_status: None,
            version: None,
            created_date: None,
            created_time: None,
            updated_date: None,
            updated_time: None,
            organization_id: None,
            created_by: None,
            updated_by: None,
            deleted_by: None,
            requested_by: None,
            tags: vec![],
            id: chrono::Utc::now().to_string(),
            timestamp: connection.connection_value.timestamp.clone(),
            interface_name: connection.connection_key.interface_name.clone().into(),
            hypertable_timestamp: None,
            total_packet: (connection.connection_value.total_packet as i32).into(),
            total_byte: (connection.connection_value.total_byte as i32).into(),
            device_id: connection.connection_key.device_id.clone().into(),
            protocol: String::from(connection.connection_key.transport_header.protocol).into(),
            source_ip: connection
                .connection_key
                .ip_header
                .source_ip
                .to_string()
                .into(),
            destination_ip: connection
                .connection_key
                .ip_header
                .destination_ip
                .to_string()
                .into(),
            remote_ip: connection
                .connection_value
                .remote_ip
                .map(|value| value.to_string()),
            source_port: connection
                .connection_key
                .transport_header
                .source_port
                .map(|value| value as i32),
            destination_port: connection
                .connection_key
                .transport_header
                .destination_port
                .map(|value| value as i32),
        };

        let request = store::CreateConnectionsRequest {
            connections: conn.into(),
            params: Some(store::CreateParams {
                table: String::from("connections"),
            }),
            query: Some(store::CreateQuery {
                pluck: String::from("id"),
                durability: String::from("hard"),
            }),
            entity_prefix: String::from("CN"),
        };

        log::debug!("Sending create connection request: {:?}", request);

        let timestamp = chrono::Utc::now();

        match ds_exp.unwrap().inner.create_connections(request).await {
            Ok(response) => {
                let diff = chrono::Utc::now() - timestamp;
                log::info!(
                    "Request to Experimental datastore took {} ms. Response: {:?}",
                    diff.num_milliseconds(),
                    response
                );
            }
            Err(err) => {
                log::error!("Request to Experimental datastore failed: {:?}", err)
            }
        }
    }
}
