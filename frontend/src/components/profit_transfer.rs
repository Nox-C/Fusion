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
    let transferring = use_state(|| false);
    let status = use_state(|| None::<String>);
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
                        status.set(Some("Error contacting server.".to_string()));
                    }
                    transferring.set(false);
                }
            });
        })
    };
    html! {
        <div class="profit-transfer-panel" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;overflow-x:auto;">
            <h3 style="color:#00ff99;">{ "Transfer Profit" }</h3>
            <div style="margin-bottom:1rem;">
                <label style="color:#fff;">{"Profit amount to transfer: "}</label>
                <input
                    type="number"
                    value={profit.to_string()}
                    oninput={{
                        let profit = profit.clone();
                        Callback::from(move |e: InputEvent| {
                            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                if let Ok(val) = input.value().parse::<f64>() {
                                    profit.set(val);
                                }
                            }
                        })
                    }}
                    min="0.0"
                    step="0.0001"
                    style="margin-left:0.5rem;padding:0.5rem 1rem;border-radius:6px;border:1px solid #444;background:#181c20;color:#fff;width:8rem;"
                />
            </div>
            <button onclick={onclick} disabled={*transferring} style="background:#00ff99;color:#23272f;font-weight:600;padding:0.75rem 2rem;border:none;border-radius:8px;font-size:1.1rem;box-shadow:0 2px 8px #00ff9940;cursor:pointer;transition:background 0.2s;">{
                if *transferring { "Transferring..." } else { "Transfer Profit" }
            }</button>
            if let Some(status) = &*status {
                <div style="margin-top:1rem;color:#00ff99;">{ status }</div>
            }
        </div>
    }
}
