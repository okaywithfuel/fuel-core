use crate::{
    graphql_metrics::GRAPHQL_METRICS,
    p2p_metrics::P2P_METRICS,
    services::SERVICES_METRICS,
    txpool_metrics::TXPOOL_METRICS,
};
use axum::{
    body::Body,
    response::{
        IntoResponse,
        Response,
    },
};
use core::ops::Deref;
use libp2p_prom_client::encoding::text::encode as libp2p_encode;
use prometheus_client::encoding::text::encode;

pub fn encode_metrics_response() -> impl IntoResponse {
    // encode libp2p metrics using older prometheus
    let mut libp2p_bytes = Vec::<u8>::new();
    if let Some(value) = P2P_METRICS.gossip_sub_registry.get() {
        if libp2p_encode(&mut libp2p_bytes, value).is_err() {
            return error_body()
        }
    }
    if libp2p_encode(&mut libp2p_bytes, &P2P_METRICS.peer_metrics).is_err() {
        return error_body()
    }

    let mut encoded = String::from_utf8_lossy(&libp2p_bytes).into_owned();

    {
        let lock = SERVICES_METRICS
            .registry
            .lock()
            .expect("The service metrics lock is poisoned");
        if encode(&mut encoded, lock.deref()).is_err() {
            return error_body()
        }
    }

    // encode the rest of the fuel-core metrics using latest prometheus
    if encode(&mut encoded, &TXPOOL_METRICS.registry).is_err() {
        return error_body()
    }

    if encode(&mut encoded, &GRAPHQL_METRICS.registry).is_err() {
        return error_body()
    }

    Response::builder()
        .status(200)
        .body(Body::from(encoded))
        .unwrap()
}

fn error_body() -> Response<Body> {
    Response::builder()
        .status(503)
        .body(Body::from(""))
        .unwrap()
}
