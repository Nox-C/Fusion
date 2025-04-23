// Tests for REST and WebSocket API endpoints

use actix_web::{App, test, web};
use fusion::api::get_matrices;
use fusion::matrix::{Matrix, MatrixManager};
use std::sync::Arc;

#[actix_web::test]
async fn test_api_matrices_endpoint() {
    // Initialize MatrixManager with default matrix
    let matrix_manager = Arc::new(MatrixManager::new());
    // Build test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(matrix_manager.clone()))
            .service(get_matrices),
    )
    .await;
    // Send request to /api/matrices
    let req = test::TestRequest::get().uri("/api/matrices").to_request();
    let resp: Vec<Matrix> = test::call_and_read_body_json(&app, req).await;
    // Verify response contains at least the default ETH matrix
    assert!(!resp.is_empty());
    assert_eq!(resp[0].chain, "ETH");
}
