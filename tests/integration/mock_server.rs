use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockRollupServer {
    notices: Arc<Mutex<Vec<String>>>,
    reports: Arc<Mutex<Vec<String>>>,
}

impl MockRollupServer {
    pub fn new() -> Self {
        Self {
            notices: Arc::new(Mutex::new(Vec::new())),
            reports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start(&self) -> String {
        let notices = self.notices.clone();
        let reports = self.reports.clone();

        let make_svc = make_service_fn(move |_conn| {
            let notices = notices.clone();
            let reports = reports.clone();

            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req, notices.clone(), reports.clone())
                }))
            }
        });

        // Bind to random port
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = Server::bind(&addr).serve(make_svc);
        let actual_addr = server.local_addr();

        // Spawn server in background
        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("Mock server error: {}", e);
            }
        });

        format!("http://{}", actual_addr)
    }

    pub fn get_notices(&self) -> Vec<String> {
        self.notices.lock().unwrap().clone()
    }

    pub fn get_reports(&self) -> Vec<String> {
        self.reports.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.notices.lock().unwrap().clear();
        self.reports.lock().unwrap().clear();
    }
}

async fn handle_request(
    req: Request<Body>,
    notices: Arc<Mutex<Vec<String>>>,
    reports: Arc<Mutex<Vec<String>>>,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    match path {
        "/notice" => {
            // Read body
            let body_bytes = hyper::body::to_bytes(req.into_body())
                .await
                .unwrap_or_default();

            if let Ok(body_str) = std::str::from_utf8(&body_bytes) {
                // Parse JSON to extract payload
                if let Ok(json) = json::parse(body_str) {
                    if let Some(payload_hex) = json["payload"].as_str() {
                        // Decode hex to get actual JSON
                        if let Ok(payload_bytes) = hex::decode(payload_hex) {
                            if let Ok(payload_json) = std::str::from_utf8(&payload_bytes) {
                                notices.lock().unwrap().push(payload_json.to_string());
                            }
                        }
                    }
                }
            }

            Ok(Response::new(Body::from("{\"status\":\"ok\"}")))
        }
        "/report" => {
            // Read body
            let body_bytes = hyper::body::to_bytes(req.into_body())
                .await
                .unwrap_or_default();

            if let Ok(body_str) = std::str::from_utf8(&body_bytes) {
                // Parse JSON to extract payload
                if let Ok(json) = json::parse(body_str) {
                    if let Some(payload_hex) = json["payload"].as_str() {
                        // Decode hex to get actual JSON
                        if let Ok(payload_bytes) = hex::decode(payload_hex) {
                            if let Ok(payload_json) = std::str::from_utf8(&payload_bytes) {
                                reports.lock().unwrap().push(payload_json.to_string());
                            }
                        }
                    }
                }
            }

            Ok(Response::new(Body::from("{\"status\":\"ok\"}")))
        }
        _ => {
            let mut response = Response::new(Body::from("Not Found"));
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_starts() {
        let server = MockRollupServer::new();
        let url = server.start().await;

        assert!(url.starts_with("http://127.0.0.1:"));
    }

    #[tokio::test]
    async fn test_mock_server_captures_notices() {
        let server = MockRollupServer::new();
        let url = server.start().await;

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Send a test notice
        let client = hyper::Client::new();
        let test_payload = r#"{"test":"notice"}"#;
        let payload_hex = hex::encode(test_payload);

        let req = hyper::Request::builder()
            .method("POST")
            .uri(format!("{}/notice", url))
            .header("content-type", "application/json")
            .body(Body::from(format!(r#"{{"payload":"{}"}}"#, payload_hex)))
            .unwrap();

        client.request(req).await.unwrap();

        // Give time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let notices = server.get_notices();
        assert_eq!(notices.len(), 1);
        assert!(notices[0].contains("test"));
    }
}
