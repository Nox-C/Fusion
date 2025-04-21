// /home/user/Fusion/src/config.rs

use serde::Deserialize;


// --- Helper function to parse comma-separated strings ---
fn parse_comma_separated_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(s.split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty()) // Avoid empty strings if there are trailing commas etc.
        .collect())
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    // Provider Priority
    pub provider_priority_order: Vec<String>,

    // --- Infura ---
    pub infura_api_key: String,
    pub rpc_url: String,
    pub bsc_rpc_url: String,
    pub bsc_testnet_rpc_url: String,
    pub eth_rpc_url: String,
    pub eth_sepolia_rpc_url: String,
    pub infura_eth_websocket_url: String,
    pub infura_eth_sepolia_websocket_url: String,
    pub infura_bsc_websocket_url: String,

    // --- Alchemy ---
    pub alchemy_api_key: String,
    pub alchemy_bsc_rpc_url: String,
    pub alchemy_eth_rpc_url: String,
    pub alchemy_bsc_testnet_rpc_url: String,
    pub alchemy_eth_sepolia_rpc_url: String,
    pub alchemy_eth_websocket_url: String,
    pub alchemy_eth_sepolia_websocket_url: String,
    pub alchemy_bsc_websocket_url: String,
    pub alchemy_bsc_testnet_websocket_url: String,

    // --- NodeReal ---
    pub nodereal_api_key: String,
    pub nodereal_bsc_rpc_url: String,
    pub nodereal_eth_rpc_url: String,
    pub nodereal_bsc_testnet_rpc_url: String,
    pub nodereal_eth_sepolia_rpc_url: String,
    pub nodereal_eth_websocket_url: String,
    pub nodereal_eth_sepolia_websocket_url: String,
    pub nodereal_bsc_websocket_url: String,
    pub nodereal_bsc_testnet_websocket_url: String,

    // --- DEX WebSockets ---
    pub websocket_pancakeswap: String,
    pub websocket_biswap: String,
    pub websocket_mdex: String,
    pub websocket_babyswap: String,
    pub websocket_apeswap: String,
    pub websocket_kokoswap: String,
    pub websocket_thena: String,
    pub websocket_waultswap: String,
    pub websocket_dodo: String,
    pub websocket_ellipsis: String,

    // --- Time Synchronization & Validation ---
    pub ntp_server: String,
    pub timestamp_sync_interval_ms: u64,
    pub max_timestamp_deviation_ms: u64,
    pub timestamp_validation_enabled: bool,
    pub data_freshness_threshold_ms: u64,
    pub clock_drift_correction: bool,

    // --- Deployment Keys ---
    pub private_key: String,
    pub profit_wallet: String,

    // --- Liquidity & Flash Loan Optimization ---
    pub liquidity_usage_percentage: f64,
    pub dynamic_liquidity_provider: bool,
    pub liquidity_provider_comparison: bool,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub flash_loan_providers: Vec<String>,
    pub aave_pool_address_provider: String,
    pub dydx_solo_margin_address: String,
    pub balancer_vault_address: String,
    pub uniswap_v3_factory: String,
    pub pancakeswap_v2_factory: String,
    pub liquidity_cache_ttl_ms: u64,
    pub liquidity_threshold_minimum_usd: f64, // Changed to f64 for potential decimals
    pub refresh_liquidity_interval_ms: u64,
    pub calculate_impermanent_loss: bool,
    pub slippage_estimation_enabled: bool,
    pub max_liquidity_utilization_stable: f64,
    pub max_liquidity_utilization_major: f64,
    pub max_liquidity_utilization_alt: f64,
    pub max_liquidity_utilization_meme: f64,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub liquidity_source_priority_order: Vec<String>,
    pub smart_order_routing: bool,

    // --- Arbitrage Parameters ---
    pub marginal_optimizer: f64,
    pub min_profit_usd: f64,
    pub max_slippage: f64,
    pub gas_price_buffer: i64, // Assuming integer Gwei or similar, verify unit
    pub gas_price_buffer_percentage: f64,

    // --- Parallel Processing Configuration ---
    pub worker_threads: u32,
    pub processing_queue_size: usize,
    pub matrix_update_interval_ms: u64,
    pub price_staleness_threshold_ms: u64,
    pub opportunity_staleness_threshold_ms: u64,
    pub max_concurrent_price_checks: usize,
    pub transaction_pre_validation: bool,
    pub concurrent_matrix_processing: bool,

    // --- Gas Settings ---
    pub max_fee_per_gas: u64,
    pub max_priority_fee_per_gas: u64,
    pub gas_estimator: String,
    pub gas_price_update_interval_ms: u64,

    // --- DEX Router Addresses ---
    pub router_pancakeswap: String,
    pub router_biswap: String,
    pub router_mdex: String,
    pub router_babyswap: String,
    pub router_apeswap: String,
    pub router_kokoswap: String,
    pub router_thena: String,
    pub router_waultswap: String,
    pub router_dodo: String,
    pub router_ellipsis: String,

    // --- DEXes to Monitor ---
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub dexes: Vec<String>,

    // --- Token Definitions ---
    // Note: Token addresses are strings. Consider using a dedicated Address type later (e.g., from ethers-rs)
    pub token_wbnb: String,
    pub token_cake: String,
    pub token_bake: String,
    pub token_xvs: String,
    pub token_sxp: String,
    pub token_alpaca: String,
    pub token_bsw: String,
    pub token_baby: String,
    pub token_bscpads: String, // Renamed from BSCPAD for consistency if needed, check .env
    pub token_busd: String,
    pub token_usdt: String,
    pub token_usdc: String,
    pub token_dai: String,
    // pub token_ust: String, // Keep commented or remove if not used
    pub token_tusd: String,
    pub token_frax: String,
    pub token_vai: String,
    pub token_mim: String,
    pub token_usdp: String,
    pub token_eth: String,
    pub token_btcb: String,
    pub token_dot: String,
    pub token_ada: String,
    pub token_xrp: String,
    pub token_sol: String,
    pub token_avax: String,
    pub token_matic: String,
    pub token_atom: String,
    pub token_near: String,
    pub token_ftm: String,
    pub token_trx: String,
    pub token_ltc: String,
    pub token_fil: String,
    pub token_link: String,
    pub token_uni: String,
    pub token_aave: String,
    pub token_comp: String,
    pub token_mkr: String,
    pub token_snx: String,
    pub token_1inch: String,
    pub token_crv: String,
    pub token_yfi: String,
    pub token_sushi: String,
    pub token_doge: String,
    pub token_shib: String,
    pub token_floki: String,
    pub token_babydoge: String,
    pub token_safemoon: String,
    pub token_cate: String,
    pub token_elongate: String,
    pub token_lowb: String,
    pub token_safemars: String,

    // --- Matrix Configurations ---
    // These might be better handled by parsing them into more structured data later,
    // but for now, we load them as strings/numbers as defined.
    pub matrix1_name: String,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix1_tokens: Vec<String>,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix1_pairs: Vec<String>,
    pub matrix1_update_priority: u32,
    pub matrix1_marginal_optimizer: f64,
    pub matrix1_update_interval_ms: u64,
    pub matrix1_timestamp_validation: bool,
    pub matrix1_liquidity_check: bool,
    pub matrix1_max_liquidity_utilization: f64,

    pub matrix2_name: String,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix2_tokens: Vec<String>,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix2_pairs: Vec<String>,
    pub matrix2_update_priority: u32,
    pub matrix2_marginal_optimizer: f64,
    pub matrix2_update_interval_ms: u64,
    pub matrix2_timestamp_validation: bool,
    pub matrix2_liquidity_check: bool,
    pub matrix2_max_liquidity_utilization: f64,

    pub matrix3_name: String,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix3_tokens: Vec<String>,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix3_pairs: Vec<String>,
    pub matrix3_update_priority: u32,
    pub matrix3_marginal_optimizer: f64,
    pub matrix3_update_interval_ms: u64,
    pub matrix3_timestamp_validation: bool,
    pub matrix3_liquidity_check: bool,
    pub matrix3_max_liquidity_utilization: f64,

    pub matrix4_name: String,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix4_tokens: Vec<String>,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix4_pairs: Vec<String>,
    pub matrix4_update_priority: u32,
    pub matrix4_marginal_optimizer: f64,
    pub matrix4_update_interval_ms: u64,
    pub matrix4_timestamp_validation: bool,
    pub matrix4_liquidity_check: bool,
    pub matrix4_max_liquidity_utilization: f64,

    pub matrix5_name: String,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix5_tokens: Vec<String>,
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub matrix5_pairs: Vec<String>,
    pub matrix5_update_priority: u32,
    pub matrix5_marginal_optimizer: f64,
    pub matrix5_update_interval_ms: u64,
    pub matrix5_timestamp_validation: bool,
    pub matrix5_liquidity_check: bool,
    pub matrix5_max_liquidity_utilization: f64,

    // --- Pre-Execution Validation ---
    #[serde(deserialize_with = "parse_comma_separated_string")]
    pub pre_execution_checks: Vec<String>,
    pub transaction_max_age_ms: u64,
    pub transaction_assembly_max_time_ms: u64,
    pub mempool_monitoring: bool,
    // pub gas_price_buffer_percentage: f64, // Already defined above
    pub simulate_transaction_before_sending: bool,

    // --- Performance Optimization ---
    pub use_shared_memory: bool,
    pub price_cache_enabled: bool,
    pub price_cache_ttl_ms: u64,
    pub batch_websocket_requests: bool,
    pub prioritize_high_volume_pairs: bool,
    pub dynamic_polling_intervals: bool,

    // --- Transaction Execution ---
    pub max_pending_transactions: u32,
    pub transaction_timeout_ms: u64,
    pub transaction_confirmation_blocks: u64,
    pub auto_retry_failed_transactions: bool,
    pub minimum_profitable_amount_usd: f64,

    // --- Verification ---
    pub etherscan_api_key: Option<String>, // Use Option for optional keys

    // --- Logging ---
    pub log_level: String,
    pub price_update_log: bool,
    pub opportunity_log: bool,
    pub execution_log: bool,
    pub error_log: bool,
    pub matrix_log: bool,
    pub timing_log: bool,
    pub websocket_log: bool,
    pub liquidity_log: bool,
    pub log_rotation_size_mb: u64,
    pub log_retention_days: u64,
    pub performance_metrics_enabled: bool,
    pub metrics_interval_ms: u64,

    // --- Provider Rotation ---
    pub provider_rotation_enabled: bool,
    pub provider_rotation_interval_ms: u64,
}

impl Settings {}

