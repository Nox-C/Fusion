// Tests for REST and WebSocket API endpoints

use actix_web::{App, test, web};
use fusion::api::get_matrix2d;
use fusion::matrix2d::Matrix2D;
use std::sync::{Arc, Mutex};

#[actix_web::test]
async fn test_api_matrix2d_endpoint() {
    // Create a sample Matrix2D and wrap in Arc<Mutex<>>
    let dexes = vec!["PancakeSwap".to_string(), "Biswap".to_string()];
    let assets = vec!["WBNB".to_string(), "BUSD".to_string()];

    let matrix = Arc::new(Mutex::new(Matrix2D::new(dexes.clone(), assets.clone())));
    // Build test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(matrix.clone()))
            .service(get_matrix2d)
    ).await;
    // Send request to /api/matrix2d
    let req = test::TestRequest::get().uri("/api/matrix2d").to_request();
    let raw = test::call_and_read_body(&app, req).await;

    let resp: Result<Matrix2D, _> = serde_json::from_slice(&raw);
    match resp {
        Ok(matrix) => {
            assert_eq!(matrix.dexes, dexes);
            assert_eq!(matrix.assets, assets);
        }
        Err(e) => panic!("Failed to parse JSON: {:?} (raw: {:?})", e, raw),
    }
}
