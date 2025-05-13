use crate::{datastore::DatastoreWrapper, ssh_keypair::SSHKeypair};
use nullnet_libdatastore::{CreateBody, CreateParams, CreateRequest, Query};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl DatastoreWrapper {
    pub async fn create_ssh_keypair_if_not_exists(
        &self,
        device_id: &str,
        token: &str,
    ) -> Result<(), Error> {
        if self
            .fetch_ssh_keypair_for_device(device_id, token)
            .await
            .is_some()
        {
            return Ok(());
        }

        let keypair = SSHKeypair::generate().await?;

        let request = CreateRequest {
            params: Some(CreateParams {
                table: String::from("device_ssh_keys"),
            }),
            query: Some(Query {
                pluck: String::from("id"),
                durability: String::from("hard"),
            }),
            body: Some(CreateBody {
                record: serde_json::to_string(&keypair).handle_err(location!())?,
                entity_prefix: String::from("SSH"),
            }),
        };

        let _ = self.inner.clone().create(request, token).await?;
        
        Ok(())
    }
}
