use std::{collections::HashMap, io, net::SocketAddr, sync::{atomic::AtomicBool, Arc}, time::Duration};

use lifx_lan::{deserialize_lifx_packet, serialize_lifx_packet, Message};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{net::UdpSocket, sync::RwLock, task::JoinHandle, time::sleep};

use crate::{Light, Request};

const DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0:56700";

pub fn create_socket(lights: Arc<RwLock<HashMap<String, Arc<RwLock<Light>>>>>, is_terminating: Arc<AtomicBool>) -> (std::sync::mpsc::Sender<Request>, JoinHandle<()>) {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

    sock.set_nonblocking(true).unwrap();
    sock.set_broadcast(true).unwrap();
    sock.set_reuse_address(true).unwrap();
    sock.set_reuse_port(true).unwrap();

    sock.set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();

    let addr: SocketAddr = DEFAULT_LISTEN_ADDRESS.parse().unwrap();
    sock.bind(&addr.into()).unwrap();

    let socket = UdpSocket::from_std(sock.into()).unwrap();
    let (tx, rx) = std::sync::mpsc::channel::<Request>();

    let handle = tokio::spawn(handle_socket(socket, rx, lights, is_terminating));
    log::info!("Socket handler thread started.");

    return (tx, handle);
}

pub async fn handle_socket(socket: UdpSocket, rx: std::sync::mpsc::Receiver<Request>, lights: Arc<RwLock<HashMap<String, Arc<RwLock<Light>>>>>, is_terminating: Arc<AtomicBool>) {
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
                                    let lights = lights.read().await;

                                    if lights.contains_key(&src.to_string()) {
                                        continue;
                                    }
                                }

                                let mut lights = lights.write().await;

                                if !lights.contains_key(&src.to_string()) {
                                    lights.insert(
                                        src.to_string(),
                                        Arc::new(RwLock::new(Light::default())),
                                    );
                                }
                            }
                        }
                        Message::Label { label } => {
                            log::debug!("Got label from {}: {}", src, label);

                            let mut lights = lights.write().await;

                            if let Some(light) = lights.get_mut(&src.to_string()) {
                                light.write().await.label = Some(label);
                            } else {
                                lights.insert(
                                    src.to_string(),
                                    Arc::new(RwLock::new(Light {
                                        label: Some(label),
                                        ..Light::default()
                                    })),
                                );
                            }
                        }
                        Message::HostFirmware {
                            build,
                            version_minor,
                            version_major,
                            ..
                        } => {
                            let mut lights = lights.write().await;

                            if let Some(light) = lights.get_mut(&src.to_string()) {
                                light.write().await.firmware_version = Some(format!(
                                    "{}.{}.{}",
                                    build, version_major, version_minor
                                ));
                            } else {
                                lights.insert(
                                    src.to_string(),
                                    Arc::new(RwLock::new(Light {
                                        firmware_version: Some(format!(
                                            "{}.{}.{}",
                                            build, version_major, version_minor
                                        )),
                                        ..Light::default()
                                    })),
                                );
                            }
                        }
                        Message::LightState {
                            hue,
                            saturation,
                            brightness,
                            kelvin,
                            power,
                            label,
                            ..
                        } => {
                            let mut lights = lights.write().await;

                            if let Some(light) = lights.get_mut(&src.to_string()) {
                                let mut light = light.write().await;

                                light.label = Some(label);
                                light.hue = Some(hue);
                                light.saturation = Some(saturation);
                                light.brightness = Some(brightness);
                                light.kelvin = Some(kelvin);
                                light.power = Some(power);
                            } else {
                                lights.insert(
                                    src.to_string(),
                                    Arc::new(RwLock::new(Light {
                                        label: Some(label),
                                        hue: Some(hue),
                                        saturation: Some(saturation),
                                        brightness: Some(brightness),
                                        kelvin: Some(kelvin),
                                        power: Some(power),
                                        ..Light::default()
                                    })),
                                );
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

            match rx.try_recv() {
                Ok(request) => {
                    serialize_lifx_packet(
                        &request.options,
                        &request.message,
                        &mut request_buffer,
                    );

                    match socket.send_to(&request_buffer, &request.target).await {
                        Ok(_) => {
                            log::debug!("Sent message to {}: {:?}", &request.target, &request.message);
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
}