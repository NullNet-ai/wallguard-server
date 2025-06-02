use tonic::{Request, Response, Status};

use crate::control_service::WallGuardService;
use crate::protocol::wallguard_authorization::AuthorizationRequest;
use crate::protocol::wallguard_service::wall_guard_server::WallGuard;

impl WallGuardService {
    pub(crate) async fn device_authorization_impl(
        &self,
        request: Request<AuthorizationRequest>,
    ) -> Result<Response<<WallGuardService as WallGuard>::DeviceAuthorizationStream>, Status> {
        todo!()
    }
}
