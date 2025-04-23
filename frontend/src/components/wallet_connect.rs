use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window};
// Removed gloo and Rc imports

#[derive(Clone, PartialEq, Debug)]
pub struct WalletState {
    pub connected: bool,
    pub address: Option<String>,
    pub network: Option<String>,
    pub error: Option<String>,
}

fn get_ethereum() -> Option<js_sys::Object> {
    let win = window()?;
    let eth_val = js_sys::Reflect::get(&win, &JsValue::from_str("ethereum")).ok()?;
    eth_val.dyn_into::<js_sys::Object>().ok()
}

#[function_component(WalletConnect)]
pub fn wallet_connect() -> Html {
    let state = use_state(|| WalletState {
        connected: false,
        address: None,
        network: None,
        error: None,
    });
    let loading = use_state(|| false);

    // Helper: connect wallet
    let connect_wallet = {
        let state = state.clone();
        let loading = loading.clone();
        Callback::from(move |_| {
            loading.set(true);
            let state = state.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let eth = js_sys::Reflect::get(&window().unwrap(), &JsValue::from_str("ethereum"));
                if let Ok(eth) = eth {
                    let req = js_sys::Reflect::get(&eth, &JsValue::from_str("request"));
                    if let Ok(req_fn) = req {
                        let req_fn = req_fn.dyn_ref::<js_sys::Function>().unwrap();
                        let params = js_sys::Object::new();
                        js_sys::Reflect::set(&params, &JsValue::from_str("method"), &JsValue::from_str("eth_requestAccounts")).unwrap();
                        match req_fn.call1(&eth, &params) {
                            Ok(_) => {
                                // After request, get accounts
                                let accs = js_sys::Reflect::get(&eth, &JsValue::from_str("selectedAddress")).ok().and_then(|v| v.as_string());
                                let net = js_sys::Reflect::get(&eth, &JsValue::from_str("networkVersion")).ok().and_then(|v| v.as_string());
                                state.set(WalletState {
                                    connected: accs.is_some(),
                                    address: accs,
                                    network: net,
                                    error: None,
                                });
                            },
                            Err(e) => {
                                state.set(WalletState {
                                    connected: false,
                                    address: None,
                                    network: None,
                                    error: Some(format!("MetaMask error: {}", js_sys::JsString::from(e))),
                                });
                            }
                        }
                    } else {
                        state.set(WalletState {
                            connected: false,
                            address: None,
                            network: None,
                            error: Some("MetaMask 'request' method not found".to_string()),
                        });
                    }
                } else {
                    state.set(WalletState {
                        connected: false,
                        address: None,
                        network: None,
                        error: Some("MetaMask not detected. Please install MetaMask.".to_string()),
                    });
                }
                loading.set(false);
            });
        })
    };

    // Effect: listen for account/network changes
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            if let Some(eth) = get_ethereum() {
                let on_accounts = Closure::<dyn FnMut(js_sys::Array)>::new({
                    let state = state.clone();
                    move |accs: js_sys::Array| {
                        let addr = accs.get(0).as_string();
                        state.set(WalletState {
                            connected: addr.is_some(),
                            address: addr,
                            network: state.network.clone(),
                            error: None,
                        });
                    }
                });
                let on_chain = Closure::<dyn FnMut(JsValue)>::new({
                    let state = state.clone();
                    move |chain: JsValue| {
                        let net = chain.as_string();
                        state.set(WalletState {
                            connected: state.connected,
                            address: state.address.clone(),
                            network: net,
                            error: None,
                        });
                    }
                });
                if let Ok(on) = js_sys::Reflect::get(&eth, &JsValue::from_str("on")) {
                    if let Some(f) = on.dyn_ref::<js_sys::Function>() {
                        let _ = f.call2(&eth, &JsValue::from_str("accountsChanged"), on_accounts.as_ref());
                        let _ = f.call2(&eth, &JsValue::from_str("chainChanged"), on_chain.as_ref());
                    }
                }
                on_accounts.forget();
                on_chain.forget();
            }
            || ()
        });
    }

    html! {
        <div class="wallet-connect" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;text-align:center;max-width:400px;margin:auto;">
            if *loading {
                <div style="color:#00ff99;opacity:0.8;padding:1rem 0;">{"Checking wallet status..."}</div>
            } else if let Some(err) = &state.error {
                <div style="color:#ff5555;font-weight:bold;padding:0.5rem 0;">{ err }</div>
                <button onclick={connect_wallet} style="background:#00ff99;color:#23272f;font-weight:bold;padding:0.75rem 2rem;border:none;border-radius:8px;font-size:1rem;box-shadow:0 2px 8px #00ff9940;cursor:pointer;transition:background 0.2s;">{"Connect Wallet"}</button>
            } else if state.connected {
                <div style="color:#00ff99;font-weight:bold;font-size:1.2rem;">{"Wallet Connected"}</div>
                <div style="color:#fff;margin-top:0.5rem;">
                    <span style="font-family:monospace;">{ state.address.as_deref().unwrap_or("Unknown") }</span>
                    <button onclick={ {
                        let addr = state.address.clone();
                        Callback::from(move |_| {
                            if let Some(addr) = &addr {
                                if let Some(win) = web_sys::window() {
                                    if let Ok(nav) = js_sys::Reflect::get(&win, &JsValue::from_str("navigator")) {
                                        if let Ok(clip) = js_sys::Reflect::get(&nav, &JsValue::from_str("clipboard")) {
                                            if let Some(clipboard) = clip.dyn_ref::<js_sys::Object>() {
                                                if let Ok(write_text_fn) = js_sys::Reflect::get(clipboard, &JsValue::from_str("writeText")) {
                                                    if let Some(write_text_fn) = write_text_fn.dyn_ref::<js_sys::Function>() {
                                                        let _ = write_text_fn.call1(clipboard, &JsValue::from_str(addr));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        })
                    }}
                    style="margin-left:0.5rem;background:none;border:none;color:#00ff99;cursor:pointer;font-size:1rem;">{"Copy"}</button>
                </div>
                <div style="color:#bbb;margin-top:0.5rem;font-size:0.95rem;">{ format!("Network: {}", state.network.as_deref().unwrap_or("Unknown")) }</div>
            } else {
                <button onclick={connect_wallet} style="background:#00ff99;color:#23272f;font-weight:bold;padding:0.75rem 2rem;border:none;border-radius:8px;font-size:1rem;box-shadow:0 2px 8px #00ff9940;cursor:pointer;transition:background 0.2s;">{"Connect Wallet"}</button>
            }
        </div>
    }
}
