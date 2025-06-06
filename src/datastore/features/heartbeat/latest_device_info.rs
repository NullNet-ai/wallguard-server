use crate::{datastore::features::utils::map_status_value_to_enum, proto::wallguard::DeviceStatus};
use nullnet_libdatastore::ResponseData;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

pub struct LatestDeviceInfo {
    pub status: DeviceStatus,
    pub is_monitoring_enabled: bool,
    pub is_packet_capture_enabled: bool,
    pub is_resource_monitoring_enabled: bool,
}

impl LatestDeviceInfo {
    pub fn from_response_data(response: &ResponseData) -> Result<Self, Error> {
        let json =
            serde_json::from_str::<serde_json::Value>(&response.data).handle_err(location!())?;
        Self::from_json(&json)
    }

    fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        let object = value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.as_object())
            .ok_or("Unexpected response data format")
            .handle_err(location!())?;

        let is_monitoring_enabled = object
            .get("is_monitoring_enabled")
            .and_then(serde_json::Value::as_bool)
            .ok_or("Could not parse 'is_monitoring_enabled'")
            .handle_err(location!())?;

        let is_packet_capture_enabled = object
            .get("is_packet_capture_enabled")
            .and_then(serde_json::Value::as_bool)
            .ok_or("Could not parse 'is_packet_capture_enabled'")
            .handle_err(location!())?;

        let is_resource_monitoring_enabled = object
            .get("is_resource_monitoring_enabled")
            .and_then(serde_json::Value::as_bool)
            .ok_or("Could not parse 'is_resource_monitoring_enabled'")
            .handle_err(location!())?;

        let status = object
            .get("status")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or("Could not parse 'status'")
            .handle_err(location!())?;

        Ok(Self {
            status: map_status_value_to_enum(&status),
            is_monitoring_enabled,
            is_packet_capture_enabled,
            is_resource_monitoring_enabled,
        })
    }
}
