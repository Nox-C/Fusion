use yew::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct Profit {
    pub amount: f64,
}

#[function_component(ProfitTransfer)]
pub fn profit_transfer() -> Html {
    let profit = use_state(|| 0.0f64);
    let loading = use_state(|| true);
    let transferring = use_state(|| false);
    let status = use_state(|| None::<String>);
    {
        let profit = profit.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local({
                let profit = profit.clone();
                let loading = loading.clone();
                async move {
                    let resp = gloo_net::http::Request::get("/api/profit")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<Profit>().await {
                            profit.set(data.amount);
                        }
                    }
                    loading.set(false);
                }
            });
            || ()
        });
    }
    let onclick = {
        let transferring = transferring.clone();
        let status = status.clone();
        let profit = profit.clone();
        Callback::from(move |_| {
            transferring.set(true);
            status.set(None);
            spawn_local({
                let transferring = transferring.clone();
                let status = status.clone();
                let profit = *profit;
                async move {
                    let resp = gloo_net::http::Request::post("/api/transfer")
                        .header("Content-Type", "application/json")
                        .body(format!("{{\"amount\":{}}}", profit))
                        .unwrap()
                        .send()
                        .await;
                    if let Ok(r) = resp {
                        if r.ok() {
                            status.set(Some("Success! Profit transferred.".to_string()));
                        } else {
                            status.set(Some("Failed to transfer.".to_string()));
                        }
                    } else {
                        status.set(Some("Error: No response.".to_string()));
                    }
                    transferring.set(false);
                }
            });
        })
    };
    html! {
        <div class="profit-transfer" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;text-align:center;">
            <h4 style="color:#00ff99;">{"Available Profit"}</h4>
            if *loading {
                <div style="color:#00ff99;opacity:0.8;padding:1rem 0;">{"Loading..."}</div>
            } else {
                <div style="font-size:1.5rem;color:#fff;margin-bottom:1rem;">{ format!("{:.4} ETH", *profit) }</div>
                <button onclick={onclick} disabled={*transferring || *profit == 0.0} style="background:#00ff99;color:#23272f;font-weight:bold;padding:0.75rem 2rem;border:none;border-radius:8px;font-size:1rem;box-shadow:0 2px 8px #00ff9940;cursor:pointer;transition:background 0.2s;">{
                    if *transferring { "Transferring..." } else { "Transfer Profit to Wallet" }
                }</button>
            }
            if let Some(msg) = &*status {
                <div style="margin-top:1rem;color:#00ff99;">{ msg }</div>
            }
        </div>
    }
}
