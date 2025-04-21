use std::collections::HashMap;

use axum::{body::Body, extract::{Query, State}, Json};
use lifx_lan::{LifxRequestOptions, Message};
use serde::Deserialize;

use crate::{onboard::send_onboarding_request, web::AppState, Light, Request};

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

#[derive(Deserialize)]
pub struct OnboardingRequest {
    ssid: String,
    password: String,
}

pub async fn trigger_onboarding(state: State<AppState>, body: Json<OnboardingRequest>) {
    log::debug!("Onboarding request");

    let ssid = body.ssid.clone();
    let password = body.password.clone();

    send_onboarding_request(ssid, password).unwrap();
}

#[derive(Deserialize)]
pub struct NameRequest {
    ip: String,
    name: String,
}

pub async fn set_name(state: State<AppState>, body: Json<NameRequest>) {
    log::debug!("Set name request for {}", body.ip);

    let lights = state.lights.read().await;
    
    let mut light = lights.get(&body.ip).unwrap().write().await;
    
    light.label = Some(body.name.clone());

    drop(light);
    drop(lights);

    let mut guard = state.last_req_sequence.lock().await;

    let sequence = *guard;
    *guard = guard.wrapping_add(1);

    drop(guard);

    // pad name to 32 bytes with null bytes
    let mut label = body.name.clone();
    label.push_str(&"\x00".repeat(32 - body.name.len()));

    state.tx.send(Request {
        options: LifxRequestOptions {
            tagged: true,
            source: 0,
            target: [0; 8],
            ack_required: false,
            res_required: true,
            sequence,
        },
        message: Message::SetLabel { label },
        target: body.ip.clone(),
    }).unwrap();
    
    return;
}