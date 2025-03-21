#![allow(clippy::module_name_repetitions)]

use datastore::DatastoreWrapper;

mod datastore;
mod grpc_server;
mod http_server;
mod parser;
mod proto;
mod utils;

#[tokio::main]
async fn main() {
    // disable logging to datastore until we have an account for authenticating server to log
    // let datastore_logger_config =
    //     nullnet_liblogging::DatastoreConfig::new("account_id", "account_secret", ADDR, PORT);
    let logger_config = nullnet_liblogging::LoggerConfig::new(true, false, None, vec![]);
    nullnet_liblogging::Logger::init(logger_config);

    let datastore = DatastoreWrapper::new()
        .await
        .expect("Failed to connect to the datastore");

    tokio::join!(
        grpc_server::run_grpc_server(datastore),
        http_server::run_http_server()
    );
}
