use ethers::prelude::*;
use ethers::abi::Abi;
use std::sync::Arc;

/// Calls the ArbitrageExecutor contract's executeArbitrage function on BSC mainnet.
pub async fn execute_arbitrage_onchain(
    contract_address: Address,
    abi_path: &str,
    flashloan_provider: Address,
    loan_token: Address,
    loan_amount: U256,
    routers: Vec<Address>,
    swap_paths: Vec<Vec<Address>>,
    amounts_in: Vec<U256>,
    amounts_out_min: Vec<U256>,
    private_key: &str,
    rpc_url: &str,
) -> Result<TxHash, Box<dyn std::error::Error>> {
    // Load ABI
    let abi = std::fs::read_to_string(abi_path)?;
    let abi: Abi = serde_json::from_str(&abi)?;

    // Setup provider and wallet
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet = private_key.parse::<LocalWallet>()?;
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // Instantiate contract
    let contract = Contract::new(contract_address, abi, client);

    // Call the contract
    let method = contract
        .method::<_, ()>(
            "executeArbitrage",
            (
                flashloan_provider,
                loan_token,
                loan_amount,
                routers,
                swap_paths,
                amounts_in,
                amounts_out_min,
            ),
        )?;
    let pending_tx = method.send().await?;
    Ok(*pending_tx)
}
