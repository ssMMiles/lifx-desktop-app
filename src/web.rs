use std::{collections::HashMap, sync::{mpsc::Sender, Arc}};

use axum::{extract::Request, http::{header, HeaderValue}, middleware::{self, Next}, response::Response, routing::{get, post}, Router};
use tokio::sync::{Mutex, RwLock};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{routes::{color, get_lights, power, set_name, trigger_onboarding}, Light};

#[derive(Clone)]
pub struct AppState {
    pub lights: Arc<RwLock<HashMap<String, Arc<RwLock<Light>>>>>,
    pub tx: Sender<crate::Request>,

    pub last_req_sequence: Arc<Mutex<u8>>,
}

pub async fn start_webserver(tx: std::sync::mpsc::Sender<crate::Request>, lights: Arc<RwLock<HashMap<String, Arc<RwLock<Light>>>>>) {
    let state = AppState {
        lights: lights.clone(),
        tx: tx.clone(),
        last_req_sequence: Arc::new(Mutex::new(0)),
    };

    let app: Router = Router::new()
        .nest_service("/", ServiceBuilder::new()
            .layer(middleware::from_fn(set_static_cache_control))
            .service(
                ServeDir::new("static/",
            ))
        )
        .route("/api/lights", get(get_lights))
        .route("/api/setPower", post(power))
        .route("/api/setColor", post(color))
        .route("/api/setName", post(set_name))
        .route("/api/onboard", post(trigger_onboarding))
        .layer(
            CorsLayer::permissive(),
        )
        .with_state(state);

    let address = std::env::var("WEB_LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("WEB_LISTEN_PORT").unwrap_or_else(|_| "3000".to_string());

    log::info!("Starting webserver on http://{}:{}", address, port);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", address, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn set_static_cache_control(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache"),
    );
    response
}