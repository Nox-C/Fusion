use fusion::matrix2d::{Matrix2D};
use fusion::analysis::scan_matrix2d;

fn main() {
    let dexes = vec!["PancakeSwap".to_string(), "Biswap".to_string(), "ApeSwap".to_string()];
    let assets = vec!["WBNB".to_string(), "BUSD".to_string(), "USDT".to_string()];
    let mut matrix = Matrix2D::new(dexes, assets);
    // Simulate price updates
    matrix.update_price("PancakeSwap", "WBNB", 600.12);
    matrix.update_price("Biswap", "WBNB", 600.10);
    matrix.update_price("ApeSwap", "WBNB", 600.25);
    matrix.update_price("PancakeSwap", "BUSD", 1.0);
    matrix.update_price("Biswap", "BUSD", 0.995);
    matrix.update_price("ApeSwap", "BUSD", 1.01);
    // Scan for arbitrage with 0.5% threshold
    let opps = scan_matrix2d(&matrix, 0.5);
    for (buy_dex, asset, sell_dex, buy_price, _, sell_price, profit_pct, ts) in opps {
        println!("[OPP] Buy {} on {} at {} | Sell on {} at {} | Profit: {:.2}% @ {}", asset, buy_dex, buy_price, sell_dex, sell_price, profit_pct, ts);
    }
}
