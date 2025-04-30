


// #[derive(Clone, Debug)]
// pub enum Dex {
//     PancakeSwap,
//     Biswap,
//     ApeSwap,
//     MDEX,
//     BabySwap,
//     DODO,
//     Thena,
//     Ellipsis,
//     WaultSwap,
// }

// /// Fetch token price from a DEX HTTP API. Returns price in quote token (e.g., BUSD per WBNB)
// pub async fn fetch_price(dex: Dex, token_address: &str) -> Option<f64> {
//     use log::info;
//     use log::warn;
//
//     let client = Client::new();
//     info!("[PRICE_FETCH] Attempting to fetch price for {:?} / {}", dex, token_address);
//     match dex {
//         Dex::PancakeSwap => {
//             // PancakeSwap API: https://api.pancakeswap.info/api/v2/tokens/{token_address}
//             let url = format!("https://api.pancakeswap.info/api/v2/tokens/{}", token_address);
//             info!("[PRICE_FETCH] PancakeSwap URL: {}", url);
//             let resp = match client.get(&url).send().await {
//                 Ok(r) => r,
//                 Err(e) => {
//                     warn!("[PRICE_FETCH][ERROR] PancakeSwap fetch error for {}: {}", url, e);
//                     return None;
//                 }
//             };
//             let text = resp.text().await.unwrap_or_else(|e| {
//                 warn!("[PRICE_FETCH][ERROR] PancakeSwap response text error: {}", e);
//                 "".to_string()
//             });
//             info!("[PRICE_FETCH] PancakeSwap raw response: {}", text);
//             let json: serde_json::Value = serde_json::from_str(&text).ok()?;
//             let price_str = json["data"]["price"].as_str()?;
//             price_str.parse::<f64>().ok()
//         }
//         Dex::Biswap => {
//             // Biswap API: https://api.biswap.org/api/v1/token/price?address={token_address}
//             let url = format!("https://api.biswap.org/api/v1/token/price?address={}", token_address);
//             let resp = client.get(&url).send().await.ok()?;
//             let json: serde_json::Value = resp.json().await.ok()?;
//             // Biswap returns price in 'priceUsd'
//             let price = json["data"]["priceUsd"].as_str()?;
//             price.parse::<f64>().ok()
//         }
//         Dex::ApeSwap => {
//             // ApeSwap API: https://api.apeswap.finance/tokens/{token_address}
//             let url = format!("https://api.apeswap.finance/tokens/{}", token_address);
//             let resp = client.get(&url).send().await.ok()?;
//             let json: serde_json::Value = resp.json().await.ok()?;
//             let price_str = json["data"]["price"].as_str()?;
//             price_str.parse::<f64>().ok()
//         }
//         Dex::MDEX => {
//             // MDEX API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for MDEX
//             None
//         }
//         Dex::BabySwap => {
//             // BabySwap API: https://api.babyswap.finance/api/v1/token/price?address={token_address}
//             let url = format!("https://api.babyswap.finance/api/v1/token/price?address={}", token_address);
//             let resp = client.get(&url).send().await.ok()?;
//             let json: serde_json::Value = resp.json().await.ok()?;
//             let price_str = json["data"]["price"].as_str()?;
//             price_str.parse::<f64>().ok()
//         }
//         Dex::DODO => {
//             // DODO API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for DODO
//             None
//         }
//         Dex::Thena => {
//             // Thena API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for Thena
//             None
//         }
//         Dex::Ellipsis => {
//             // Ellipsis API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for Ellipsis
//             None
//         }
//         Dex::WaultSwap => {
//             // WaultSwap API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for WaultSwap
//             None
//         }
//     }
// }
// The following code was accidentally left outside the commented-out fetch_price function and should be commented out or removed to fix the delimiter error.
//             let json: serde_json::Value = resp.json().await.ok()?;
//             // Biswap returns price in 'priceUsd'
//             let price = json["data"]["priceUsd"].as_str()?;
//             price.parse::<f64>().ok()
//         }
//         Dex::ApeSwap => {
//             // ApeSwap API: https://api.apeswap.finance/tokens/{token_address}
//             let url = format!("https://api.apeswap.finance/tokens/{}", token_address);
//             let resp = client.get(&url).send().await.ok()?;
//             let json: serde_json::Value = resp.json().await.ok()?;
//             let price_str = json["data"]["price"].as_str()?;
//             price_str.parse::<f64>().ok()
//         }
//         Dex::MDEX => {
//             // MDEX API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for MDEX
//             None
//         }
//         Dex::BabySwap => {
//             // BabySwap API: https://api.babyswap.finance/api/v1/token/price?address={token_address}
//             let url = format!("https://api.babyswap.finance/api/v1/token/price?address={}", token_address);
//             let resp = client.get(&url).send().await.ok()?;
//             let json: serde_json::Value = resp.json().await.ok()?;
//             let price_str = json["data"]["price"].as_str()?;
//             price_str.parse::<f64>().ok()
//         }
//         Dex::DODO => {
//             // DODO API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for DODO
//             None
//         }
//         Dex::Thena => {
//             // Thena API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for Thena
//             None
//         }
//         Dex::Ellipsis => {
//             // Ellipsis API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for Ellipsis
//             None
//         }
//         Dex::WaultSwap => {
//             // WaultSwap API: No public HTTP price endpoint; on-chain fetch required
//             // TODO: Implement on-chain fetch for WaultSwap
//             None
//         }
//     }
// }
