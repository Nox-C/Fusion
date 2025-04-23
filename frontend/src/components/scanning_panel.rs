use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[function_component(ScanningPanel)]
pub fn scanning_panel() -> Html {
    html! {
        <div class="scanning-panel" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;overflow-x:auto;">
            <h3 style="color:#00ff99;">{ "Live Arbitrage Scanning" }</h3>
            <p style="color:#bbb;margin-top:1rem;">{"Scanning is now handled by live matrix opportunities in the dashboard. Please refer to the Matrices section for real-time updates."}</p>
        </div>
    }
}
