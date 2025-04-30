use crate::matrix2d::{Matrix2D, PriceCell};

fn main() {
    // Example DEXes and assets
    let dexes = vec!["PancakeSwap".to_string(), "Biswap".to_string(), "ApeSwap".to_string()];
    let assets = vec!["WBNB".to_string(), "BUSD".to_string(), "USDT".to_string()];

    // Initialize the matrix
    let mut matrix = Matrix2D::new(dexes, assets);

    // Simulate WebSocket price updates
    matrix.update_price("PancakeSwap", "WBNB", 600.12);
    matrix.update_price("PancakeSwap", "BUSD", 1.0);
    matrix.update_price("Biswap", "WBNB", 600.10);
    matrix.update_price("ApeSwap", "USDT", 0.999);

    // Print matrix for verification
    for (dex_idx, dex) in matrix.dexes.iter().enumerate() {
        print!("{}\t", dex);
        for asset_idx in 0..matrix.assets.len() {
            let cell = &matrix.prices[dex_idx][asset_idx];
            print!("{}@{}\t", cell.price, cell.timestamp);
        }
        println!("");
    }
}
