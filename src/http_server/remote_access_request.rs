use crate::{
    app_context::AppContext,
    tunnel::{ClientProfile, RAType},
};
use actix_web::{HttpRequest, HttpResponse, Responder, http::header::AUTHORIZATION, web};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
    ra_type: String,
}

// Request remote access
// This request does not open the session, only register a profile for a device
pub async fn remote_access_request(
    req: HttpRequest,
    context: web::Data<AppContext>,
    body: web::Json<RequestPayload>,
) -> impl Responder {
    let Ok(ra_type) = RAType::from_str(&body.ra_type) else {
        return HttpResponse::BadRequest().body("Bad ra_type parameter value");
    };

    let Some(jwt_token) = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|hv| hv.to_str().ok())
    else {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    };

    // Checking the device settings also authorizes current request.
    // While fetching data, datastore validates if the token is valid.
    let Ok(feature_enabled) = context
        .datastore
        .device_check_if_remote_access_enabled(jwt_token, body.device_id.clone())
        .await
    else {
        return HttpResponse::InternalServerError()
            .body("Failed to validate if remote access is enabled");
    };

    if !feature_enabled {
        return HttpResponse::BadRequest().body("Remote access is disabled for this device");
    }

    if context
        .tunnel
        .lock()
        .await
        .get_profile_by_device_id(&body.device_id, &ra_type)
        .await
        .is_some()
    {
        return HttpResponse::Ok().body("");
    }

    let protocol = context
        .datastore
        .device_fetch_webgui_protocol(&body.device_id, jwt_token)
        .await
        .unwrap_or(String::from("https"));

    let Ok(profile) = ClientProfile::new(&body.device_id, ra_type, protocol).await else {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    };

    if context
        .datastore
        .device_new_remote_session(
            jwt_token,
            body.device_id.clone(),
            profile.remote_access_type(),
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to save session info");
    };

    if context
        .tunnel
        .lock()
        .await
        .insert_profile(profile, ra_type)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    }

    HttpResponse::Ok().body("")
}
