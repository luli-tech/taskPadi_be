use async_nats::ConnectOptions;
use axum::{routing::post, Router, Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use futures::StreamExt;
use std::fs;

#[derive(Clone)]
struct AppState {
    nats: async_nats::Client,
}

#[derive(Deserialize, Serialize)]
struct MessagePayload {
    content: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // We will use the raw creds defined previously to ensure we can connect smoothly
    let raw_creds = r#"-----BEGIN NATS USER JWT-----
eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJPWlY3STRPTFRaN1pZTk80UVRCU1VSQVJXNUgzR09HQ1hXREJGUkNUR0hVUUVJUU1BRDNRIiwiaWF0IjoxNzcxNzk4MDc5LCJpc3MiOiJBRFlYWjY3WDVDWEY2M0xDSlBBVUZNSEYzNjcyR0ZGRkFYSEVBR0FGU1IzNVg3STZMSjVWUVBaUiIsIm5hbWUiOiJDTEkiLCJzdWIiOiJVQVBVTERVSklNS1dPR0VSNkg2RTcyUlUzN0VOUkxZRTY1NkVZUVRZMldIS0dEVlpTNkhNNEVGVCIsIm5hdHMiOnsicHViIjp7fSwic3ViIjp7fSwic3VicyI6LTEsImRhdGEiOi0xLCJwYXlsb2FkIjotMSwiaXNzdWVyX2FjY291bnQiOiJBQlpPVEpXU05DQU1RNllVUDRMRE40VEhIRVBLRlpRREFWVUhXV1U0QVFGVUg3WjZVTzZFUkxNVyIsInR5cGUiOiJ1c2VyIiwiY29kZSI6Mn19.YWsYxSnKRS8St4pFeupcwUs6Bii4X3hj40BKgHoRX5BnosLWjPPAXfAbshRPyyRAPXvSSVor6hBJ1MbhBgyzCw
------END NATS USER JWT------

************************* IMPORTANT *************************
NKEY Seed printed below can be used to sign and prove identity.
NKEYs are sensitive and should be treated as secrets.

-----BEGIN USER NKEY SEED-----
SUADYN3HVZY4CEGZAIMARZBF6XHSZASLGJPYLSDW4NXSFBPHNF4RIW3XJU
------END USER NKEY SEED------

*************************************************************"#;

    let clean_creds = raw_creds.replace("\r", "");
    
    // Write creds to a temp file, as async_nats expects a file string
    let path = std::env::temp_dir().join("nats_debug.creds");
    match fs::write(&path, &clean_creds) {
        Ok(_) => println!("✔ Written clean creds to {:?}", path),
        Err(e) => println!("❌ Failed to write: {}", e),
    }

    // Connect to Synadia Cloud via TLS
    println!("Connecting to NGS (tls://connect.ngs.global:4222)...");
    let nats = ConnectOptions::with_credentials_file(&path)
        .await?
        .connect("tls://connect.ngs.global:4222")
        .await?;
        
    println!("✔ Successfully connected to NATS Cloud!");

    // Start Worker Subscriber in background
    let nats_subscriber = nats.clone();
    tokio::spawn(async move {
        println!("👂 Worker subscriber starting...");
        
        let mut sub = match nats_subscriber.subscribe("events.messages").await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Failed to subscribe: {}", e);
                return;
            }
        };

        println!("👂 Worker listening on 'events.messages'");

        while let Some(msg) = sub.next().await {
            if let Ok(text) = std::str::from_utf8(&msg.payload) {
                println!("📦 Worker Received via Core NATS: {}", text);
            } else {
                println!("📦 Worker Received binary: {:?}", msg.payload);
            }
        }
    });

    // Create Axum Router
    let state = AppState { nats };

    let app = Router::new()
        .route("/publish", post(publish))      // Core NATS publish
        .route("/publish_js", post(publish_js)) // JetStream publish
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr).await?;
    println!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn publish(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<MessagePayload>,
) -> &'static str {
    let data = serde_json::to_vec(&payload).unwrap();

    let result = state
        .nats
        .publish("events.messages", data.into())
        .await;

    match result {
        Ok(_) => "Message published via Core NATS\n",
        Err(_) => "Failed to publish\n",
    }
}

async fn publish_js(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<MessagePayload>,
) -> &'static str {
    let data = serde_json::to_vec(&payload).unwrap();

    // Create a JetStream context from the NATS client
    let js = async_nats::jetstream::new(state.nats.clone());
    
    // Publish using JetStream
    let result = js
        .publish("events.messages", data.into())
        .await;

    match result {
        Ok(_) => "Message published via JetStream\n",
        Err(_) => "Failed to publish via JetStream\n",
    }
}
