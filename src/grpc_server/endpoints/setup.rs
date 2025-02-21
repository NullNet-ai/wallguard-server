use tonic::{Request, Response, Status};

use crate::{
    grpc_server::server::WallGuardImpl,
    proto::wallguard::{CommonResponse, SetupRequest},
};

impl WallGuardImpl {
    pub(crate) async fn setup_impl(
        &self,
        request: Request<SetupRequest>,
    ) -> Result<Response<CommonResponse>, Status> {
        let remote_address = request
            .remote_addr()
            .map_or_else(|| "Unknown".to_string(), |addr| addr.ip().to_string());

        let setup_request = request.into_inner();

        let (jwt_token, token_info) = Self::authenticate(setup_request.auth)?;

        let response = self
            .datastore
            .device_setup(
                jwt_token,
                token_info.account.device.id,
                setup_request.device_version,
                setup_request.device_uuid,
                remote_address,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if !response.success {
            return Err(Status::internal(format!(
                "Status: {}, Message: {}, Error: {}",
                response.status_code, response.message, response.error
            )));
        }

        Ok(Response::new(CommonResponse {
            success: response.success,
            message: response.message,
        }))
    }
}
