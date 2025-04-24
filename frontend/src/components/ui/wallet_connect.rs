use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use js_sys::Array;
use js_sys::Function;
use js_sys::Object;
use web_sys::window;
use crate::components::ui::button::Button;

#[derive(Clone, PartialEq, Debug)]
pub struct WalletState {
    pub connected: bool,
    pub address: Option<String>,
    pub network: Option<String>,
    pub error: Option<String>,
}

fn get_ethereum() -> Option<Object> {
    let win = window()?;
    let eth = js_sys::Reflect::get(&win, &JsValue::from_str("ethereum")).ok()?;
    eth.dyn_into::<Object>().ok()
}

#[function_component(WalletConnect)]
pub fn wallet_connect() -> Html {
    let state = use_state(|| WalletState { connected: false, address: None, network: None, error: None });
    let loading = use_state(|| false);

    let onclick = {
        let state = state.clone();
        let loading = loading.clone();
        Callback::from(move |_| {
            loading.set(true);
            let state = state.clone();
            let loading = loading.clone();
            spawn_local(async move {
                if let Some(eth) = get_ethereum() {
                    if let Ok(req) = js_sys::Reflect::get(&eth, &JsValue::from_str("request")) {
                        if let Some(req_fn) = req.dyn_ref::<Function>() {
                            let params = Object::new();
                            js_sys::Reflect::set(&params, &JsValue::from_str("method"), &JsValue::from_str("eth_requestAccounts")).unwrap();
                            if req_fn.call1(&eth, &params).is_ok() {
                                let addr = js_sys::Reflect::get(&eth, &JsValue::from_str("selectedAddress")).ok().and_then(|v| v.as_string());
                                let net = js_sys::Reflect::get(&eth, &JsValue::from_str("networkVersion")).ok().and_then(|v| v.as_string());
                                state.set(WalletState { connected: addr.is_some(), address: addr, network: net, error: None });
                            } else {
                                state.set(WalletState { connected: false, address: None, network: None, error: Some("MetaMask request failed".to_string()) });
                            }
                        }
                    }
                } else {
                    state.set(WalletState { connected: false, address: None, network: None, error: Some("MetaMask not found".to_string()) });
                }
                loading.set(false);
            });
        })
    };

    // Listen for account and chain changes
    {
        let state = state.clone();
        use_effect(move || {
            if let Some(eth) = get_ethereum() {
                // accountsChanged
                let state_accounts = state.clone();
                let on_accounts = Closure::<dyn FnMut(Array)>::new(Box::new(move |accounts: Array| {
                    let addr = accounts.get(0).as_string();
                    state_accounts.set(WalletState {
                        connected: addr.is_some(),
                        address: addr,
                        network: (*state_accounts).network.clone(),
                        error: None,
                    });
                }));
                // chainChanged
                let state_chain = state.clone();
                let on_chain = Closure::<dyn FnMut(JsValue)>::new(Box::new(move |chain: JsValue| {
                    let net = chain.as_string();
                    state_chain.set(WalletState {
                        connected: (*state_chain).connected,
                        address: (*state_chain).address.clone(),
                        network: net,
                        error: None,
                    });
                }));
                if let Ok(on_fn) = js_sys::Reflect::get(&eth, &JsValue::from_str("on")) {
                    if let Some(f) = on_fn.dyn_ref::<Function>() {
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
        <div class="bg-[hsl(var(--card))] p-6 rounded-lg shadow-lg mb-6 max-w-md mx-auto text-center">
            if *loading {
                <p class="text-muted-foreground">{"Connecting to wallet..."}</p>
            } else if let Some(err) = &state.error {
                <p class="text-destructive">{ err }</p>
                <Button class="mt-2 bg-primary text-white px-4 py-2 rounded" onclick={onclick.clone()}>
                    { "Connect Wallet" }
                </Button>
            } else if state.connected {
                <p class="text-[hsl(var(--success))] font-bold">{"Wallet Connected"}</p>
                <p class="font-mono break-all">{ state.address.clone().unwrap_or_default() }</p>
                <p class="text-muted-foreground">{ format!("Network: {}", state.network.clone().unwrap_or_default()) }</p>
                <Button class="mt-2 bg-primary text-white px-4 py-2 rounded" onclick={onclick.clone()}>
                    { "Reconnect" }
                </Button>
            } else {
                <Button class="bg-primary text-white px-4 py-2 rounded" onclick={onclick.clone()}>
                    { "Connect Wallet" }
                </Button>
            }
        </div>
    }
}
