// /home/nox/Fusion/src/main.rs

mod config;

use std::sync::Arc;
use fusion::matrix::MatrixManager;

use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Fusion backend API...");
    let matrix_manager = Arc::new(MatrixManager::new());
    HttpServer::new(move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(matrix_manager.clone()))
            .service(fusion::api::get_matrices)
            .service(fusion::api::get_scanning)
            .service(fusion::api::get_completed_transactions)
            .service(fusion::api::get_marginal_optimizer)
            .service(fusion::api::set_marginal_optimizer)
            .service(fusion::api::get_liquidity)
            .service(fusion::api::set_liquidity)
            .service(fusion::api::get_profit)
            .service(fusion::api::post_transfer)
            .service(fusion::api::get_wallet_status)
            .service(fusion::api::post_connect_wallet)
            .service(fusion::api::get_flashloan_providers)
            .route("/ws/matrices", actix_web::web::get().to(fusion::api::ws_matrices_handler))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
