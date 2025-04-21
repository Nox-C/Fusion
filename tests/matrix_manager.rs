// Integration and unit tests for MatrixManager and matrix logic
use fusion::matrix::{Matrix, MatrixManager};

#[test]
fn test_matrix_creation_and_serialization() {
    let m = Matrix {
        id: "matrix1".to_string(),
        name: "ETH Matrix".to_string(),
        chain: "ETH".to_string(),
        marginal_optimizer: 0.0,
        dex_prices: std::collections::HashMap::new(),
        opportunities: vec![],
        recent_transactions: vec![],
        status: "Active".to_string(),
    };
    let serialized = serde_json::to_string(&m).unwrap();
    let deserialized: Matrix = serde_json::from_str(&serialized).unwrap();
    assert_eq!(m.id, deserialized.id);
}

#[test]
fn test_matrix_manager_all() {
    let manager = MatrixManager::new();
    let all = manager.all();
    assert!(!all.is_empty());
    assert!(all.iter().any(|m| m.chain == "ETH" || m.chain == "BSC"));
}
