use std::collections::HashMap;

use axum::{extract::{Query, State}, response::Html, Json};
use lifx_lan::{LifxRequestOptions, Message};
use serde::Deserialize;

use crate::{web::AppState, Light, Request};

pub async fn get_lights(state: State<AppState>) -> Json<HashMap<String, Light>> {
    let lights_read_guard = state.lights.read().await;
    
    let mut lights = HashMap::new();

    for (addr, light) in lights_read_guard.iter() {
        let light = light.read().await;

        lights.insert(addr.clone(), (*light).clone());
    }

    drop(lights_read_guard);

    Json(lights)
}

#[derive(Deserialize)]
pub struct PowerRequest {
    ip: String,
}

// take IP address as query parameter
pub async fn power(state: State<AppState>, query: Query<PowerRequest>) {
    log::debug!("Toggle power request for {}", query.ip);

    let lights = state.lights.read().await;
    
    let mut light = lights.get(&query.ip).unwrap().write().await;
    
    let new_power = if light.power == Some(65535) {
        0
    } else {
        65535
    };

    light.power = Some(new_power);

    drop(light);
    drop(lights);

    let mut guard = state.last_req_sequence.lock().await;

    let sequence = *guard;
    *guard = guard.wrapping_add(1);

    drop(guard);

    state.tx.send(Request {
        options: LifxRequestOptions {
            tagged: true,
            source: 0,
            target: [0; 8],
            ack_required: false,
            res_required: true,
            sequence,
        },
        message: Message::SetPower {
            level: new_power
        },
        target: query.ip.clone(),
    }).unwrap(); 
}

#[derive(Deserialize)]
pub struct ColorRequest {
    ip: String,

    hue: u16,
    saturation: u16,
    brightness: u16,

    kelvin: u16,
}

pub async fn color(state: State<AppState>, query: Query<ColorRequest>) {
    log::debug!("Color request for {}", query.ip);

    let lights = state.lights.read().await;
    
    let mut light = lights.get(&query.ip).unwrap().write().await;
    
    light.hue = Some(query.hue);
    light.saturation = Some(query.saturation);
    light.brightness = Some(query.brightness);
    light.kelvin = Some(query.kelvin);

    drop(light);
    drop(lights);

    let mut guard = state.last_req_sequence.lock().await;

    let sequence = *guard;
    *guard = guard.wrapping_add(1);

    drop(guard);

    state.tx.send(Request {
        options: LifxRequestOptions {
            tagged: true,
            source: 0,
            target: [0; 8],
            ack_required: false,
            res_required: true,
            sequence,
        },
        message: Message::SetColor {
            reserved_6: 1,
            hue: query.hue,
            saturation: query.saturation,
            brightness: query.brightness,
            kelvin: query.kelvin,
            duration_ms: 450,
        },
        target: query.ip.clone(),
    }).unwrap(); 
}
