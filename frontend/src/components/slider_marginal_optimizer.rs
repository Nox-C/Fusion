// Placeholder for slider marginal optimizer component

use yew::prelude::*;
use serde::{Deserialize, Serialize};
// Note: spawn_local imported but unused; kept for consistency if needed
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct MarginalOptimizer {
    pub threshold: f64,
}

#[function_component(SliderMarginalOptimizer)]
pub fn slider_marginal_optimizer() -> Html {
    let threshold = use_state(|| 1.0f64);

    let oninput = {
        let threshold = threshold.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            let value = input.value().parse::<f64>().unwrap_or(1.0);
            threshold.set(value);
        })
    };
    html! {
        <div class="slider-marginal-optimizer" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;">
            <label style="color:#00ff99;font-weight:bold;">{ "Marginal Optimizer Threshold (%): " }
                <span style="margin-left:1rem;color:#fff;">{ format!("{:.2}", *threshold) }</span>
            </label>
            <input type="range" min="0.5" max="10.0" step="0.1" value={threshold.to_string()} oninput={oninput} style="width:100%;accent-color:#00ff99;" />
            <div style="color:#bbb;margin-top:1rem;font-size:0.98rem;">{"Note: Optimizer threshold is now managed by the backend in production."}</div>
        </div>
    }
}
