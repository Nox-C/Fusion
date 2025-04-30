// Integration and unit tests for MatrixManager and matrix logic
use fusion::matrix2d::Matrix2D;

#[test]
fn test_matrix2d_creation_and_serialization() {
    let dexes = vec!["PancakeSwap".to_string(), "Biswap".to_string()];
    let assets = vec!["WBNB".to_string(), "BUSD".to_string()];
    let matrix = Matrix2D::new(dexes.clone(), assets.clone());
    // Check dimensions
    assert_eq!(matrix.dexes, dexes);
    assert_eq!(matrix.assets, assets);
    assert_eq!(matrix.prices.len(), 2);
    assert_eq!(matrix.prices[0].len(), 2);
    // Serialization
    let serialized = serde_json::to_string(&matrix).unwrap();
    let deserialized: Matrix2D = serde_json::from_str(&serialized).unwrap();
    assert_eq!(matrix, deserialized);
}

#[test]
fn test_matrix2d_update_and_query() {
    let dexes = vec!["PancakeSwap".to_string(), "Biswap".to_string()];
    let assets = vec!["WBNB".to_string(), "BUSD".to_string()];
    let mut matrix = Matrix2D::new(dexes.clone(), assets.clone());
    matrix.update_price("PancakeSwap", "WBNB", 600.12);
    let price_cell = matrix.get_price("PancakeSwap", "WBNB");
    assert!(price_cell.is_some());
    assert_eq!(price_cell.unwrap().price, 600.12);
}
