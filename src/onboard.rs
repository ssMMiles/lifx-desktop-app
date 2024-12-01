extern crate lifx_lan;

use std::{io::{self, Write}, net::{SocketAddr, TcpStream}, str::FromStr, time::Duration};

// use heapless::String;

use lifx_lan::{messages::Message, request_options::LifxRequestOptions, serialize_lifx_packet};
use native_tls::TlsConnector;

fn pad_with_nulls(input: &str, target_length: usize) -> String {
    let mut padded = String::from_str(input).unwrap();
    while padded.len() < target_length {
        // Append null bytes until the string reaches the target length
        padded.push('\0'); // Use unwrap since we are sure of the capacity
    }
    padded
}

pub fn send_onboarding_request(mut ssid: String, mut password: String) -> Result<(), io::Error> {
    ssid = pad_with_nulls(&ssid, 32);
    password = pad_with_nulls(&password, 64);

    let light_address: SocketAddr = "172.16.0.1:56700".parse().unwrap();

    let mut message_buffer = [0u8; 36 + 32 + 64 + 2];

    let req_options = LifxRequestOptions {
        tagged: true,
        source: 0,
        target: [0; 8],
        ack_required: true,
        res_required: true,
        sequence: 0,
    };

    println!("Sending onboarding request with SSID: {} and password: {}", ssid, password);

    serialize_lifx_packet(&req_options, &Message::SetAccessPoint { 
        interface: 2,
        ssid, 
        password,
        protocol: 5
    }, &mut message_buffer);

    let tcp_stream = TcpStream::connect_timeout(&light_address, Duration::from_secs(3))?;

    tcp_stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    tcp_stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();

    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true) 
        .danger_accept_invalid_hostnames(true) 
        .build()
        .expect("Failed to build TLS connector");

    let mut tls_stream = connector
        .connect("hostname-does-not-matter", tcp_stream)
        .expect("TLS handshake failed");

    tls_stream.write_all(&message_buffer).unwrap();

    println!("Onboarding request sent.");

    tls_stream.shutdown().unwrap();

    return Ok(());
}