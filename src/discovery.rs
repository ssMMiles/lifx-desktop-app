use std::sync::{atomic::AtomicBool, Arc};

use lifx_lan::{LifxRequestOptions, Message};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};

use crate::Request;

const GET_COLOR_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);
const DISCOVERY_REQUEST_INTERVAL_FACTOR: u32 = 5;

pub async fn broadcast_discovery_requests(tx: std::sync::mpsc::Sender<Request>, is_terminating: Arc<AtomicBool>) {
    let mut req_options = LifxRequestOptions {
        tagged: true,
        source: 10,
        target: [0; 8],
        ack_required: false,
        res_required: true,
        sequence: 0,
    };

    let mut count_since_last_discovery = 0;

    loop {
        if is_terminating.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        
        let network_interfaces = NetworkInterface::show().unwrap();

        for interface in &network_interfaces {
            for addr in &interface.addr {
                match addr {
                    Addr::V4(v4addr) => {
                        if let Some(broadcast_address) = v4addr.broadcast {
                            if broadcast_address.to_string() == "172.16.0.255" {
                                // lifx bulb's own network, skip
                                continue;
                            }

                            let target = format!("{}:56700", broadcast_address);
                            // if count_since_last_discovery == DISCOVERY_REQUEST_INTERVAL_FACTOR {
                            //     tx.send(Request {
                            //         options: req_options.clone(),
                            //         message: Message::GetColor,
                            //         target: target.clone(),
                            //     })
                            //     .unwrap();
                        
                        
                            //     // tx.send(Request {
                            //     //     options: req_options.clone(),
                            //     //     message: Message::GetHostFirmware,
                            //     //     target: target.clone(),
                            //     // })
                            //     // .unwrap();

                            //     // req_options.increment_sequence();
                            // }

                            tx.send(Request {
                                options: req_options.clone(),
                                message: Message::GetColor,
                                target: target.clone(),
                            })
                            .unwrap();
                            req_options.increment_sequence();
                        }
                    }
                    _ => {}
                }
            }
        }

        if count_since_last_discovery == DISCOVERY_REQUEST_INTERVAL_FACTOR {
            count_since_last_discovery = 0;
        } else {
            count_since_last_discovery += 1;
        }

        tokio::time::sleep(GET_COLOR_INTERVAL).await;
    }
}