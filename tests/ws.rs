// Integration test for the /ws/matrices WebSocket endpoint
use actix_web::{web, App, HttpServer};
use actix_rt;
use awc::ws::{Frame, Message};
use awc::Client;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Assuming these are the correct paths based on your project structure
use fusion::api_ws::ws_matrix2d_handler;
use fusion::matrix2d::Matrix2D;
use fusion::events::{WebSocketEvent, DexEvent, LiquidationEvent};

// Helper function to set up the test server with broadcast channel
async fn setup_test_server() -> (TcpListener, actix_rt::task::JoinHandle<()>, tokio::sync::broadcast::Sender<WebSocketEvent>) {
    // Bind to a random available port on localhost
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let addr = listener.local_addr().expect("Failed to get local address");
    println!("Test server listening on: {}", addr);

    // Create shared state
    let matrix_data: Arc<Mutex<Matrix2D>> =
        Arc::new(Mutex::new(Matrix2D::new(vec!["Uniswap".into()], vec!["ETH".into()])));

    // Create broadcast channel
    let (event_tx, _) = tokio::sync::broadcast::channel::<WebSocketEvent>(100);

    // Start the server in a background task
    let server_handle = actix_rt::spawn(async move {
        println!("Starting test server...");
        HttpServer::new(move || {
            let app_data = web::Data::from(matrix_data.clone());
            let event_tx = web::Data::new(event_tx.clone());
            App::new()
                .app_data(app_data)
                .app_data(event_tx)
                .route("/ws/matrix2d", web::get().to(ws_matrix2d_handler))
        })
        .listen(listener)
        .expect("Failed to listen")
        .run()
        .await
        .unwrap();
    });

    // Give server time to start
    actix_rt::time::sleep(Duration::from_millis(100)).await;
    (listener, server_handle, event_tx)
}

// Helper function to connect WebSocket client
async fn connect_ws_client(addr: &std::net::SocketAddr) -> (awc::ClientResponse, awc::ws::WsFramed<awc::Client>) {
    let url = format!("ws://{}", addr);
    let connect_url = format!("{}/ws/matrix2d", url);
    println!("Connecting WebSocket client to: {}", connect_url);

    let client_connect_result = Client::new().ws(&connect_url).connect().await;
    match client_connect_result {
        Ok(connection) => {
            println!("WebSocket client connected successfully.");
            connection
        }
        Err(e) => panic!("Failed to connect WebSocket client: {:?}", e),
    }
}

#[actix_web::test]
async fn test_ws_basic_connection_and_echo() {
    // Set up server
    let (listener, server_handle) = setup_test_server().await;
    let addr = listener.local_addr().unwrap();

    // Connect client
    let (_response, mut framed) = connect_ws_client(&addr).await;

    // 1. Verify Welcome Message
    println!("Waiting for welcome message...");
    if let Some(Ok(frame)) = framed.next().await {
        match frame {
            Frame::Text(txt_bytes) => {
                let txt = String::from_utf8_lossy(&txt_bytes);
                println!("Received message: {}", txt);
                assert!(txt.contains("Welcome"), "Welcome message mismatch");
            }
            _ => panic!("Expected a Text frame for welcome message, got: {:?}", frame),
        }
    } else {
        server_handle.abort();
        panic!("Did not receive any frame for welcome message (stream ended or error)");
    }

    // 2. Test Echo Functionality
    let echo_message = "Hello Actix!";
    println!("Sending echo message: {}", echo_message);
    if let Err(e) = framed.send(Message::Text(echo_message.into())).await {
        server_handle.abort();
        panic!("Failed to send echo message: {:?}", e);
    }

    println!("Waiting for echo response...");
    if let Some(Ok(frame)) = framed.next().await {
        match frame {
            Frame::Text(txt_bytes) => {
                let txt = String::from_utf8_lossy(&txt_bytes);
                println!("Received message: {}", txt);
                let expected_echo = format!("Echo: {}", echo_message);
                assert!(txt.contains(&expected_echo), "Echo response mismatch");
            }
            _ => panic!("Expected a Text frame for echo response, got: {:?}", frame),
        }
    } else {
        server_handle.abort();
        panic!("Did not receive any frame for echo response (stream ended or error)");
    }

    // Clean up
    server_handle.abort();
    actix_rt::time::sleep(Duration::from_millis(50)).await;
}

#[actix_web::test]
async fn test_ws_event_streaming() {
    // Set up server
    let (listener, server_handle) = setup_test_server().await;
    let addr = listener.local_addr().unwrap();

    // Connect client
    let (_response, mut framed) = connect_ws_client(&addr).await;

    // 1. Verify Welcome Message
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        assert!(String::from_utf8_lossy(&txt).contains("Welcome"));
    } else {
        panic!("Did not receive welcome message");
    }

    // 2. Test server-initiated DEX event streaming
    // Send a DEX event through the broadcast channel
    let expected_dex_event = json!({
        "type": "dex",
        "id": "test-event-id-123",
        "timestamp": "2025-05-01T10:26:53Z",
        "dex": "Uniswap",
        "message": "Large ETH trade detected",
        "status": "active",
        "created_at": "2025-05-01T10:26:53Z"
    });

    // Send the event through the broadcast channel
    event_tx.send(WebSocketEvent::Dex(DexEvent {
        id: "test-event-id-123".to_string(),
        timestamp: "2025-05-01T10:26:53Z".to_string(),
        dex: "Uniswap".to_string(),
        message: "Large ETH trade detected".to_string(),
        status: "active".to_string(),
        created_at: "2025-05-01T10:26:53Z".to_string(),
    })).unwrap();

    // Wait for and verify the server-initiated event
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let received_event: serde_json::Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert_eq!(received_event, expected_dex_event, "Received event doesn't match expected format");
    } else {
        panic!("Did not receive server-initiated DEX event");
    }

    // 3. Test server-initiated liquidation event streaming
    let expected_liquidation_event = json!({
        "type": "liquidation",
        "id": "test-liquidation-id-456",
        "timestamp": "2025-05-01T10:26:53Z",
        "account": "0x1234567890123456789012345678901234567890",
        "status": "active",
        "details": "High-risk position detected",
        "created_at": "2025-05-01T10:26:53Z"
    });

    // Send the liquidation event through the broadcast channel
    event_tx.send(WebSocketEvent::Liquidation(LiquidationEvent {
        id: "test-liquidation-id-456".to_string(),
        timestamp: "2025-05-01T10:26:53Z".to_string(),
        account: "0x1234567890123456789012345678901234567890".to_string(),
        status: "active".to_string(),
        details: "High-risk position detected".to_string(),
        created_at: "2025-05-01T10:26:53Z".to_string(),
    })).unwrap();

    // Wait for and verify the server-initiated liquidation event
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let received_event: serde_json::Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert_eq!(received_event, expected_liquidation_event, "Received liquidation event doesn't match expected format");
    } else {
        panic!("Did not receive server-initiated liquidation event");
    }

    // Clean up
    server_handle.abort();
    actix_rt::time::sleep(Duration::from_millis(50)).await;
}

#[actix_web::test]
async fn test_ws_event_validation() {
    // Set up server
    let (listener, server_handle) = setup_test_server().await;
    let addr = listener.local_addr().unwrap();

    // Connect client
    let (_response, mut framed) = connect_ws_client(&addr).await;

    // Verify welcome message
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        assert!(String::from_utf8_lossy(&txt).contains("Welcome"));
    } else {
        panic!("Did not receive welcome message");
    }

    // Test unknown event type
    let unknown_event = json!({
        "type": "unknown",
        "data": "some data"
    });

    framed
        .send(Message::Text(unknown_event.to_string()))
        .await
        .expect("Failed to send unknown event type");

    // Verify error response for unknown event type
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let response: Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert_eq!(response["error"].as_str(), Some("Unknown event type: unknown"));
    } else {
        panic!("Did not receive error response for unknown event type");
    }

    // Test malformed JSON
    if let Err(e) = framed.send(Message::Text("{malformed json}".into())).await {
        println!("Expected error for malformed JSON: {}", e);
    } else {
        panic!("Did not receive error for malformed JSON");
    }

    // Test liquidation event handling
    let liquidation_event = json!({
        "type": "liquidation",
        "timestamp": "2025-05-01T10:26:54Z",
        "account": "0x123...",
        "status": "completed",
        "details": "ETH liquidated on Binance"
    });

    framed
        .send(Message::Text(liquidation_event.to_string()))
        .await
        .expect("Failed to send liquidation event");

    // Verify liquidation event response
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let response: Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert_eq!(response["type"].as_str(), Some("liquidation"));
        assert_eq!(response["account"].as_str(), Some("0x123..."));
        assert_eq!(response["status"].as_str(), Some("completed"));
        assert_eq!(response["timestamp"].as_str(), Some("2025-05-01T10:26:54Z"));
        assert_eq!(response["details"].as_str(), Some("ETH liquidated on Binance"));
    } else {
        panic!("Did not receive liquidation event response");
    }

    // Test invalid liquidation event (missing required field)
    let invalid_liquidation_event = json!({
        "type": "liquidation",
        "timestamp": "2025-05-01T10:26:54Z",
        "status": "completed",
        "details": "ETH liquidated on Binance"
    });

    framed
        .send(Message::Text(invalid_liquidation_event.to_string()))
        .await
        .expect("Failed to send invalid liquidation event");

    // Verify error response for invalid liquidation event
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let response: Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert_eq!(response["error"].as_str(), Some("Missing required 'account' field in liquidation event"));
    } else {
        panic!("Did not receive error response for invalid liquidation event");
    }

    // Test unknown event type
    let unknown_event = json!({
        "type": "unknown",
        "data": "some data"
    });

    framed
        .send(Message::Text(unknown_event.to_string()))
        .await
        .expect("Failed to send unknown event type");

    // Verify error response for unknown event type
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let response: Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert_eq!(response["error"].as_str(), Some("Unknown event type: unknown"));
    } else {
        panic!("Did not receive error response for unknown event type");
    }

    // Test malformed JSON
    if let Err(e) = framed.send(Message::Text("{malformed json}")).await {
        println!("Expected error for malformed JSON: {}", e);
    } else {
        panic!("Did not receive error for malformed JSON");
    }
        .expect("Failed to send liquidation event");

    // Verify liquidation event is processed correctly
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let response: Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert_eq!(response["type"].as_str(), Some("liquidation"));
        assert_eq!(response["account"].as_str(), Some("0x123..."));
        assert_eq!(response["status"].as_str(), Some("completed"));
    } else {
        panic!("Did not receive liquidation event response");
    }

    // Test invalid event type
    let invalid_event = json!({
        "type": "invalid_type",
        "timestamp": "2025-05-01T10:26:55Z"
    });

    framed
        .send(Message::Text(invalid_event.to_string()))
        .await
        .expect("Failed to send invalid event");

    // Verify error handling for invalid event
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let response: Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert!(response["error"].as_str().unwrap_or("").contains("Unknown event type"));
    } else {
        panic!("Did not receive error response for invalid event");
    }

    // Test malformed JSON
    framed
        .send(Message::Text("{invalid json}".into()))
        .await
        .expect("Failed to send malformed JSON");

    // Verify error handling for malformed JSON
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        let response: Value = serde_json::from_slice(&txt).expect("Invalid JSON response");
        assert!(response["error"].as_str().unwrap_or("").contains("Invalid JSON"));
    } else {
        panic!("Did not receive error response for malformed JSON");
    }

    // Shutdown the server
    drop(srv);
}
