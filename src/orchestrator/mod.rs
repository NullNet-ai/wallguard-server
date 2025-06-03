use client::Client;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tonic::Status;

use crate::protocol::wallguard_commands::WallGuardCommand;
use crate::token_provider::TokenProvider;

mod authorization_stream;
mod client;
mod control_stream;

type ClientsMap = Arc<Mutex<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Orchestrator {
    clients: ClientsMap,
}

impl Orchestrator {
    pub fn new() -> Self {
        let clients = Arc::new(Mutex::new(HashMap::new()));

        Self { clients }
    }

    pub async fn is_client_connected(&self, device_uuid: &str) -> bool {
        self.clients.lock().await.contains_key(device_uuid)
    }

    pub async fn on_client_connected(
        &self,
        device_uuid: &str,
        token_provider: TokenProvider,
        control_stream: mpsc::Sender<Result<WallGuardCommand, Status>>,
    ) -> Result<(), Error> {
        if self.is_client_connected(device_uuid).await {
            let message = format!("Client with UUID '{device_uuid}' is already connected");
            return Err(message).handle_err(location!())?;
        }

        let client = Client::new(device_uuid, token_provider, control_stream, self.clone());

        self.clients.lock().await.insert(device_uuid.into(), client);

        Ok(())
    }

    pub async fn on_client_disconnected(&self, device_uuid: &str) -> Result<(), Error> {
        if self.clients.lock().await.remove(device_uuid).is_none() {
            Err(format!(
                "Device with UUID '{}' is not connected",
                device_uuid
            ))
            .handle_err(location!())?;
        }
        Ok(())
    }

    pub async fn is_auth_pending(&self,  device_uuid: &str) -> bool {
        false
    }

    pub async fn on_client_requested_authorization(&self, device_uuid: &str, org_id: &str) {

    }

    pub async fn on_client_authorization_completed(&self, device_uuid: &str) {

    }

    pub async fn get_client(&self, device_id: &str) -> Option<Client> {
        self.clients.lock().await.get(device_id).cloned()
    }
}
