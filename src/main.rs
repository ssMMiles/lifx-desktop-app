use std::{
    collections::HashMap, sync::{atomic::AtomicBool, Arc},
};

use lifx_lan::{messages::Message, request_options::LifxRequestOptions};

use ctrlc;
use log::{debug, info};
use serde::Serialize;
use tokio::{
    select,
    sync::RwLock,
};

extern crate socket2;

mod socket;
mod discovery;

mod web;
mod routes;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let is_terminating = Arc::new(AtomicBool::new(false));

    let is_terminating_clone = is_terminating.clone();
    ctrlc::set_handler(move || {
        debug!("Received Ctrl-C signal.");

        is_terminating_clone.store(true, std::sync::atomic::Ordering::Release);
    })
    .expect("Error setting Ctrl-C handler");

    let lights = Arc::new(RwLock::new(HashMap::<String, Arc<RwLock<Light>>>::new()));
    let (tx, socket_handle) = socket::create_socket(lights.clone(), is_terminating.clone());

    let light_discovery_handle = tokio::spawn(
        discovery::broadcast_discovery_requests(tx.clone(), is_terminating.clone())
    );
    log::info!("Started discovery thread.");

    let webserver_handle = tokio::spawn(web::start_webserver(tx.clone(), lights.clone()));
    log::info!("Webserver thread started.");

    select! {
        _ = socket_handle => {
            info!("Socket handler thread exited.");
        }
        _ = light_discovery_handle => {
            info!("Light discovery handler thread exited.");
        }
        _ = webserver_handle => {
            info!("Webserver thread exited.");
        }
    }
}

#[derive(Debug, Clone)]
struct Request {
    pub options: LifxRequestOptions,
    pub message: Message,

    pub target: String,
}

#[derive(Debug, Clone, Serialize)]
struct Light {
    pub label: Option<String>,
    pub firmware_version: Option<String>,

    pub power: Option<u16>,

    pub hue: Option<u16>,
    pub saturation: Option<u16>,
    pub brightness: Option<u16>,

    pub kelvin: Option<u16>,
}

impl Default for Light {
    fn default() -> Self {
        Light {
            label: None,
            firmware_version: None,

            power: None,

            hue: None,
            saturation: None,
            brightness: None,

            kelvin: None,
        }
    }
}