use crate::{
    app_context::AppContext,
    grpc_server::AuthHandler,
    utils::{ACCOUNT_ID, ACCOUNT_SECRET},
};

use super::{RAType, TunnelServer};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::Instant};

// @TODO: one a profile is removed, wee need to mark session in the DB as terminated

const DEFAULT_PERIOD_SECONDS: u64 = 5;

pub async fn monitor_idle_profiles(context: AppContext, timeout: Duration) {
    let mut shells: HashMap<String, Instant> = HashMap::new();
    let mut uis: HashMap<String, Instant> = HashMap::new();

    let mut auth_handler = AuthHandler::new(
        ACCOUNT_ID.to_string(),
        ACCOUNT_SECRET.to_string(),
        context.datastore.clone(),
    );

    loop {
        update_map(&mut shells, context.tunnel.clone(), RAType::Shell).await;
        update_map(&mut uis, context.tunnel.clone(), RAType::UI).await;

        remove_non_existing_profiles(&mut shells, context.tunnel.clone(), RAType::Shell).await;
        remove_non_existing_profiles(&mut uis, context.tunnel.clone(), RAType::UI).await;

        let doomed_shells = update_timestamps_and_find_doomed_profiles(
            &mut shells,
            context.tunnel.clone(),
            timeout,
            RAType::Shell,
        )
        .await;

        if !doomed_shells.is_empty() {
            if let Ok(token) = auth_handler.obtain_token_safe().await {
                terminate_profiles(
                    doomed_shells,
                    &mut shells,
                    context.clone(),
                    RAType::Shell,
                    &token,
                )
                .await;
            }
        }

        let doomed_uis = update_timestamps_and_find_doomed_profiles(
            &mut uis,
            context.tunnel.clone(),
            timeout,
            RAType::UI,
        )
        .await;

        if !doomed_uis.is_empty() {
            if let Ok(token) = auth_handler.obtain_token_safe().await {
                terminate_profiles(doomed_uis, &mut uis, context.clone(), RAType::UI, &token).await;
            }
        }

        tokio::time::sleep(Duration::from_secs(DEFAULT_PERIOD_SECONDS)).await;
    }
}

async fn remove_non_existing_profiles(
    map: &mut HashMap<String, Instant>,
    tunnel: Arc<Mutex<TunnelServer>>,
    ra_type: RAType,
) {
    let lock = tunnel.lock().await;

    let devices_to_remove: Vec<String> = map
        .keys()
        .filter(|device_id| !lock.does_profile_exist(device_id, &ra_type))
        .cloned()
        .collect();

    for device_id in devices_to_remove {
        map.remove(&device_id);
    }
}

async fn update_map(
    map: &mut HashMap<String, Instant>,
    tunnel: Arc<Mutex<TunnelServer>>,
    ra_type: RAType,
) {
    for (dev_id, _) in tunnel
        .lock()
        .await
        .devices_map
        .get(&ra_type)
        .unwrap_or(&HashMap::new())
        .iter()
    {
        if !map.contains_key(dev_id) {
            map.insert(dev_id.clone(), Instant::now());
        }
    }
}

async fn update_timestamps_and_find_doomed_profiles(
    map: &mut HashMap<String, Instant>,
    tunnel: Arc<Mutex<TunnelServer>>,
    timeout: Duration,
    ra_type: RAType,
) -> Vec<String> {
    let mut retval = vec![];

    let lock = tunnel.lock().await;

    for (device_id, timestamp) in map.iter_mut() {
        if lock
            .get_profile_if_online_by_device_id(device_id, &ra_type)
            .await
            .is_some()
        {
            *timestamp = Instant::now();
        } else if timestamp.elapsed() >= timeout {
            retval.push(device_id.to_owned());
        }
    }

    retval
}

async fn terminate_profiles(
    devices_to_remove: Vec<String>,
    map: &mut HashMap<String, Instant>,
    context: AppContext,
    ra_type: RAType,
    token: &str,
) {
    for device_id in devices_to_remove {
        log::info!(
            "Terminating Remote Session for device '{}' because it was idle for too long",
            device_id
        );

        let _ = context
            .tunnel
            .lock()
            .await
            .remove_profile(&device_id, &ra_type)
            .await;

        let _ = context
            .datastore
            .device_terminate_remote_session(token, device_id.clone(), ra_type)
            .await;

        map.remove(&device_id);
    }
}
