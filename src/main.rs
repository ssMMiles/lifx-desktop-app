use std::{
    collections::{HashMap, HashSet},
    io,
    net::SocketAddr,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
    time::Duration,
};

use axum::{extract::State, http::HeaderValue, routing::get, Json, Router};
use lifx_lan::{
    deserialize_lifx_packet, messages::Message, request_options::LifxRequestOptions,
    serialize_lifx_packet,
};

use ctrlc;
use log::{debug, error, info, log};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rand::random;
use serde::Serialize;
use std::sync::mpsc::channel;
use tokio::{
    net::UdpSocket,
    select,
    sync::{Mutex, RwLock},
    time::sleep,
};
use tower_http::cors::CorsLayer;

extern crate socket2;
use socket2::{Domain, Protocol, Socket, Type};

const LIGHT_COUNT: usize = 2;

#[derive(Debug, Clone)]
struct Request {
    pub options: LifxRequestOptions,
    pub message: Message,

    pub target: Option<String>,
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

    let local_address = "10.220.10.7:56700";
    let broadcast_address = "10.220.10.255:56700";

    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    sock.set_nonblocking(true).unwrap();
    sock.set_broadcast(true).unwrap();
    sock.set_reuse_address(true).unwrap();
    sock.set_reuse_port(true).unwrap();

    sock.set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();

    let addr: SocketAddr = local_address.parse().unwrap();
    sock.bind(&addr.into()).unwrap();

    let socket: Arc<UdpSocket> = Arc::new(UdpSocket::from_std(sock.into()).unwrap());

    let mut req_options = LifxRequestOptions {
        tagged: true,
        source: 0,
        target: [0; 8],
        ack_required: false,
        res_required: true,
        sequence: 0,
    };

    let lights = Arc::new(RwLock::new(HashMap::<String, Light>::new()));
    let (tx, rx) = channel::<Request>();

    let socket_handler_lights = lights.clone();
    let is_terminating_clone = is_terminating.clone();

    let socket_handler = tokio::spawn(async move {
        let mut socket_handler_req_options = LifxRequestOptions {
            tagged: true,
            source: 0,
            target: [0; 8],
            ack_required: false,
            res_required: true,
            sequence: 0,
        };

        let is_terminating = is_terminating_clone.clone();

        let mut message_buffer = [0u8; 128];
        let mut request_buffer = [0u8; 128];

        loop {
            loop {
                if (*is_terminating).load(std::sync::atomic::Ordering::Acquire) {
                    return;
                }

                match socket.try_recv_from(&mut message_buffer) {
                    Ok((_size, src)) => {
                        let (_header, payload) = match deserialize_lifx_packet(&message_buffer) {
                            Ok(header_payload) => header_payload,
                            Err(e) => {
                                eprintln!("Failed to deserialize packet: {}", e,);
                                continue;
                            }
                        };

                        log::debug!("Received message from {}: {:?}", src, payload);

                        match payload {
                            Message::StateService { service, port: _ } => {
                                if service == 1 {
                                    log::debug!("Got UDP Service advertisement from {}", src);

                                    {
                                        let lights = socket_handler_lights.read().await;

                                        if lights.contains_key(&src.to_string()) {
                                            continue;
                                        }
                                    }

                                    let mut lights = socket_handler_lights.write().await;

                                    if !lights.contains_key(&src.to_string()) {
                                        lights.insert(
                                            src.to_string(),
                                            Light {
                                                label: None,
                                                firmware_version: None,

                                                power: None,

                                                hue: None,
                                                saturation: None,
                                                brightness: None,

                                                kelvin: None,
                                            },
                                        );
                                    }
                                }
                            }
                            Message::Label { label } => {
                                log::debug!("Got label from {}: {}", src, label);

                                let mut lights = socket_handler_lights.write().await;

                                if let Some(light) = lights.get_mut(&src.to_string()) {
                                    light.label = Some(label);
                                }
                            }
                            Message::HostFirmware {
                                build,
                                reserved_6,
                                version_minor,
                                version_major,
                            } => {
                                let mut lights = socket_handler_lights.write().await;

                                if let Some(light) = lights.get_mut(&src.to_string()) {
                                    light.firmware_version = Some(format!(
                                        "{}.{}.{}",
                                        build, version_major, version_minor
                                    ));
                                }
                            }
                            Message::LightState {
                                hue,
                                saturation,
                                brightness,
                                kelvin,
                                reserved_6,
                                power,
                                label,
                                reserved_7,
                            } => {
                                let mut lights = socket_handler_lights.write().await;

                                if let Some(light) = lights.get_mut(&src.to_string()) {
                                    light.hue = Some(hue);
                                    light.saturation = Some(saturation);
                                    light.brightness = Some(brightness);
                                    light.kelvin = Some(kelvin);
                                    light.power = Some(power);
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(e) => {
                        eprintln!("Failed to receive a datagram: {}", e);
                        break;
                    }
                }
            }

            if (*is_terminating).load(std::sync::atomic::Ordering::Acquire) {
                return;
            }

            loop {
                if (*is_terminating).load(std::sync::atomic::Ordering::Acquire) {
                    return;
                }

                // log::debug!("Checking for outgoing messages...");

                match rx.try_recv() {
                    Ok(request) => {
                        serialize_lifx_packet(
                            &request.options,
                            &request.message,
                            &mut request_buffer,
                        );

                        let target = match request.target {
                            Some(target) => target,
                            None => broadcast_address.to_string(),
                        };

                        match socket.send_to(&request_buffer, &target).await {
                            Ok(_) => {
                                log::debug!("Sent message to {}: {:?}", &target, request.message);
                            }
                            Err(e) => eprintln!("Failed to send message: {}", e),
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            sleep(Duration::from_millis(20)).await;
        }
    });

    log::info!("Socket handler thread started.");

    let lights_clone = lights.clone();
    let discovery_tx = tx.clone();
    let light_discovery_handler = tokio::spawn(async move {
        let tx = discovery_tx;
        let lights = lights_clone;

        tx.send(Request {
            options: req_options.clone(),
            message: Message::GetService,
            target: None,
        })
        .unwrap();

        req_options.increment_sequence();

        sleep(Duration::from_secs(1)).await;

        tx.send(Request {
            options: req_options.clone(),
            message: Message::GetLabel,
            target: None,
        })
        .unwrap();

        req_options.increment_sequence();

        tx.send(Request {
            options: req_options.clone(),
            message: Message::GetHostFirmware,
            target: None,
        })
        .unwrap();

        loop {
            if (*is_terminating).load(std::sync::atomic::Ordering::Acquire) {
                break;
            }

            tx.send(Request {
                options: req_options.clone(),
                message: Message::GetColor,
                target: None,
            })
            .unwrap();

            req_options.increment_sequence();

            sleep(Duration::from_millis(800)).await;

            let lights = lights.read().await;
            for (addr, light) in lights.iter() {
                // set light brightness to + or - 10% of its prev value

                let mut brightness = match light.brightness {
                    Some(brightness) => brightness,
                    None => continue,
                };

                if brightness < 5000 {
                    brightness = 5000;
                } else if brightness > 20000 {
                    brightness = 20000;
                }

                let mut amount_to_add = (brightness as f32 * 0.08) as i32;

                if random() {
                    amount_to_add *= -1;
                }

                let new_brightness = brightness as i32 + amount_to_add;

                tx.send(Request {
                    options: req_options.clone(),
                    message: Message::SetColor {
                        reserved_6: 1,
                        hue: light.hue.unwrap(),
                        saturation: light.saturation.unwrap(),
                        brightness: new_brightness as u16,
                        kelvin: light.kelvin.unwrap(),
                        duration_ms: 450,
                    },
                    target: Some(addr.to_string()),
                })
                .unwrap();

                req_options.increment_sequence();
            }

            sleep(Duration::from_millis(350)).await;
        }
    });

    log::info!("Started discovery thread.");

    let lights_clone = lights.clone();
    let webserver = tokio::spawn(async move {
        let lights = lights_clone;
        let app: Router = Router::new()
            // `GET /` goes to `root`
            .layer(
                CorsLayer::new().allow_origin("localhochst:3002".parse::<HeaderValue>().unwrap()),
            )
            .route("/", get(root).with_state(lights.clone()));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    log::info!("Webserver thread started.");

    select! {
        _ = socket_handler => {
            info!("Socket handler thread exited.");
        }
        _ = light_discovery_handler => {
            info!("Light discovery handler thread exited.");
        }
        _ = webserver => {
            info!("Webserver thread exited.");
        }
    }
}

async fn root(state: State<Arc<RwLock<HashMap<String, Light>>>>) -> Json<HashMap<String, Light>> {
    let lights = state.read().await;

    Json((*lights).clone())
}
