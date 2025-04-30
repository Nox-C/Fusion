// Integration test for the /ws/matrices WebSocket endpoint
use awc::Client;
use actix_web::App;
use awc::ws::{Frame, Message};
use futures_util::{SinkExt, StreamExt};
use std::net::TcpListener;

use fusion::api_ws::ws_matrix2d_handler;

#[actix_web::test]
async fn test_ws_echo() {
    

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let addr = listener.local_addr().unwrap();

    // Start the server in a background task
    use std::sync::{Arc, Mutex};
    use fusion::matrix2d::Matrix2D;
    let dummy_matrix: Arc<Mutex<Matrix2D>> = Arc::new(Mutex::new(Matrix2D::new(vec!["Uniswap".into()], vec!["ETH".into()])));
    let srv = actix_rt::spawn(async move {
        actix_web::HttpServer::new(move || {
            let matrix = dummy_matrix.clone();
            App::new()
                .app_data(actix_web::web::Data::from(matrix.clone()))
                .route("/ws/matrix2d", actix_web::web::get().to(
                    move |req, stream| ws_matrix2d_handler(req, stream, actix_web::web::Data::new(matrix.clone()))
                ))
        })
        .listen(listener)
        .expect("Failed to listen")
        .run()
        .await
        .unwrap();
    });

    // Give the server a moment to start
    actix_rt::time::sleep(std::time::Duration::from_millis(100)).await;

    // Connect WebSocket client to the test server
    let url = format!("http://{}/ws/matrix2d", addr);
    let (_response, mut framed) = Client::new()
        .ws(&url)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Receive welcome message
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        assert!(String::from_utf8_lossy(&txt).contains("Welcome"));
    } else {
        panic!("Did not receive welcome message");
    }

    // Test echo functionality
    framed
        .send(Message::Text("Hello".into()))
        .await
        .expect("Send text");
    if let Some(Ok(Frame::Text(txt))) = framed.next().await {
        assert!(String::from_utf8_lossy(&txt).contains("Echo: Hello"));
    } else {
        panic!("Did not receive echo response");
    }

    // Shutdown the server
    drop(srv);
}
