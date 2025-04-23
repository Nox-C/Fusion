// Integration test for the /ws/matrices WebSocket endpoint
use actix_web_actors::ws;
use awc::Client;
use futures_util::{SinkExt, StreamExt};

#[actix_web::test]
async fn test_ws_echo() {
    use actix_web::App;
    use std::net::TcpListener;

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let addr = listener.local_addr().unwrap();

    // Start the server in a background task
    let srv = actix_rt::spawn(async move {
        actix_web::HttpServer::new(|| {
            App::new().route(
                "/ws/matrices",
                actix_web::web::get().to(fusion::api::ws_matrices_handler),
            )
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
    let url = format!("http://{}/ws/matrices", addr);
    let (_response, mut framed) = Client::new()
        .ws(&url)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Receive welcome message
    if let Some(Ok(ws::Frame::Text(txt))) = framed.next().await {
        assert!(String::from_utf8_lossy(&txt).contains("Welcome"));
    } else {
        panic!("Did not receive welcome message");
    }

    // Test echo functionality
    framed
        .send(ws::Message::Text("Hello".into()))
        .await
        .expect("Send text");
    if let Some(Ok(ws::Frame::Text(txt))) = framed.next().await {
        assert!(String::from_utf8_lossy(&txt).contains("Echo: Hello"));
    } else {
        panic!("Did not receive echo response");
    }

    // Shutdown the server
    drop(srv);
}
