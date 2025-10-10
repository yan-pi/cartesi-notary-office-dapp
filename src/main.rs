use dapp::handlers::{get_repository, handle_advance, handle_inspect};
use json::object;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Cartesi Notary DApp");

    // Test database connection
    let test_repo = get_repository();
    println!(
        "Database initialized with {} documents",
        test_repo.count_documents().unwrap_or(0)
    );
    drop(test_repo); // Close test connection

    let client = hyper::Client::new();
    let server_addr = env::var("ROLLUP_HTTP_SERVER_URL")?;

    println!("Connected to rollup server at: {}", server_addr);

    let mut status = "accept";
    loop {
        println!("Sending finish with status: {}", status);
        let response = object! {"status" => status};
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.dump()))?;
        let response = client.request(request).await?;
        println!("Received finish status {}", response.status());

        if response.status() == hyper::StatusCode::ACCEPTED {
            println!("No pending rollup request, trying again");
        } else {
            let body = hyper::body::to_bytes(response).await?;
            let utf = std::str::from_utf8(&body)?;
            let req = json::parse(utf)?;

            let request_type = req["request_type"]
                .as_str()
                .ok_or("request_type is not a string")?;

            println!("Processing request type: {}", request_type);

            status = match request_type {
                "advance_state" => handle_advance(&client, &server_addr[..], req).await?,
                "inspect_state" => handle_inspect(&client, &server_addr[..], req).await?,
                &_ => {
                    eprintln!("Unknown request type: {}", request_type);
                    "reject"
                }
            };
        }
    }
}
