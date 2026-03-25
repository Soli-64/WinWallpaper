
use std::path::Path;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SetRequest {
    path: String
}

#[derive(Deserialize,Debug)]
struct SetResponse {
    success: bool
}

pub async fn send_to_tauri_server(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request_body = SetRequest {
        path: path.to_string()
    };

    println!("Sending request to server with path: {}", path);

    let response = client
        .post("http://localhost:3001/set")
        .json(&request_body)
        .send()
        .await?;

    let result: SetResponse = response.json().await?;
    println!("Server response: {:?}", result);
    
    Ok(())
}

