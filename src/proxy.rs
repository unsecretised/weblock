use axum::{
    Json, Router,
    body::Body,
    extract::{Request, State},
    http::HeaderName,
    response::{AppendHeaders, IntoResponse, Response},
    routing::{any, post},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use openssl::rand::rand_bytes;
use reqwest::{Client, header::SET_COOKIE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::helper::extract_cookie;

#[derive(Debug, Clone)]
pub struct ServerState {
    password: String,
    jwt_secret: String,
    outport: u32,
    client: Client,
    html: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct JWTPayload {
    exp: u32,
    password: String,
}

impl Default for JWTPayload {
    fn default() -> Self {
        JWTPayload {
            exp: u32::MAX,
            password: String::new(),
        }
    }
}

impl JWTPayload {
    pub fn gen_secret() -> String {
        let mut buf = [0u8; 32];
        rand_bytes(&mut buf).expect("Unable to generate JWT secret");

        BASE64_STANDARD.encode(buf)
    }

    pub fn encode(&self, secret: String) -> String {
        let header = jsonwebtoken::Header::new(Algorithm::HS256);
        let secret = EncodingKey::from_secret(secret.as_bytes());

        jsonwebtoken::encode(&header, &self, &secret).unwrap()
    }

    pub fn decode(encoded: String, secret: String) -> Option<JWTPayload> {
        let key = DecodingKey::from_secret(secret.as_bytes());
        jsonwebtoken::decode(encoded, &key, &Validation::new(Algorithm::HS256))
            .map(|x| x.claims)
            .ok()
    }

    pub fn to_cookie(&self, secret: String) -> (HeaderName, String) {
        let jwt = self.encode(secret);
        (
            SET_COOKIE,
            format!("weblock_auth={jwt}; Path=/; SameSite=None; Secure"),
        )
    }
}

pub async fn start_proxy(inport: u32, outport: u32, password: String) {
    let jwt_secret = JWTPayload::gen_secret();
    let client = Client::new();
    let html = include_str!("../index.html").to_string();
    let state = ServerState {
        jwt_secret,
        password,
        outport,
        client,
        html,
    };

    let router = Router::new()
        .route("/weblock/signin", post(signin_handler))
        .fallback(any(proxy_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", inport))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();

    println!("Starting proxy");
}

pub async fn signin_handler(
    State(state): State<ServerState>,
    Json(payload): Json<JWTPayload>,
) -> impl IntoResponse {
    if payload.password == state.password {
        return (
            AppendHeaders(vec![payload.to_cookie(state.jwt_secret)]),
            json!({"success": true}).to_string(),
        );
    }
    return (AppendHeaders(vec![]), json!({"success": false}).to_string());
}

#[axum::debug_handler]
pub async fn proxy_handler(
    State(state): State<ServerState>,
    req: axum::http::Request<axum::body::Body>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let headers = req.headers();
    let payload = extract_cookie(headers, "weblock_auth").ok_or((
        AppendHeaders([("content-type", "text/html")]),
        state.html.clone(),
    ))?;

    let decoded = JWTPayload::decode(payload, state.jwt_secret).ok_or((
        AppendHeaders([("content-type", "text/html")]),
        state.html.clone(),
    ))?;

    if decoded.password != state.password {
        return Err((
            AppendHeaders([("content-type", "text/html")]),
            state.html.clone(),
        ));
    }

    Ok(proxy_request(req, state.outport, state.client).await)
}

pub async fn proxy_request(
    req: Request<Body>,
    outport: u32,
    client: Client,
) -> Result<Response<Body>, ()> {
    let mut headers = req.headers().clone();
    headers.remove("connection");
    headers.remove("transfer-encoding");
    headers.remove("host");

    let method = req.method().clone();
    let uri = format!(
        "http://0.0.0.0:{}{}",
        outport,
        req.uri()
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or("/")
    );

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap();

    let upstream_response = client
        .request(method, &uri)
        .headers(headers)
        .body(body_bytes)
        .send()
        .await
        .map_err(|_| ())?;

    let status = upstream_response.status();
    let headers = upstream_response.headers().clone();
    let body = upstream_response.bytes().await.unwrap();

    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    Ok(response)
}
