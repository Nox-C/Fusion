use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct WalletStatus {
    pub connected: bool,
    pub address: Option<String>,
}

#[function_component(WalletConnect)]
pub fn wallet_connect() -> Html {
    let status = use_state(|| WalletStatus { connected: false, address: None });
    let loading = use_state(|| true);
    {
        let status = status.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local({
                let status = status.clone();
                let loading = loading.clone();
                async move {
                    let resp = gloo_net::http::Request::get("/api/wallet_status")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<WalletStatus>().await {
                            status.set(data);
                        }
                    }
                    loading.set(false);
                }
            });
            || ()
        });
    }
    let onclick = {
        let status = status.clone();
        let loading = loading.clone();
        Callback::from(move |_| {
            loading.set(true);
            spawn_local({
                let status = status.clone();
                let loading = loading.clone();
                async move {
                    let resp = gloo_net::http::Request::post("/api/connect_wallet")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<WalletStatus>().await {
                            status.set(data);
                        }
                    }
                    loading.set(false);
                }
            });
        })
    };
    html! {
        <div class="wallet-connect" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;text-align:center;">
            if *loading {
                <div style="color:#00ff99;opacity:0.8;padding:1rem 0;">{"Checking wallet status..."}</div>
            } else if status.connected {
                <div style="color:#00ff99;font-weight:bold;font-size:1.2rem;">{"Wallet Connected"}</div>
                <div style="color:#fff;margin-top:0.5rem;">{ status.address.as_deref().unwrap_or("Unknown") }</div>
            } else {
                <button onclick={onclick} style="background:#00ff99;color:#23272f;font-weight:bold;padding:0.75rem 2rem;border:none;border-radius:8px;font-size:1rem;box-shadow:0 2px 8px #00ff9940;cursor:pointer;transition:background 0.2s;">{"Connect Wallet"}</button>
            }
        </div>
    }
}
