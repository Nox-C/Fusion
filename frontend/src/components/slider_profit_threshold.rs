use yew::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct MarginalOptimizer {
    pub threshold: f64,
}

#[function_component(SliderMarginalOptimizer)]
pub fn slider_marginal_optimizer() -> Html {
    let threshold = use_state(|| 1.0f64);
    let loading = use_state(|| true);
    {
        let threshold = threshold.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local({
                let threshold = threshold.clone();
                let loading = loading.clone();
                async move {
                    let resp = gloo_net::http::Request::get("/api/marginal_optimizer")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<MarginalOptimizer>().await {
                            threshold.set(data.threshold);
                        }
                    }
                    loading.set(false);
                }
            });
            || ()
        });
    }
    let oninput = {
        let threshold = threshold.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            let value = input.value().parse::<f64>().unwrap_or(1.0);
            threshold.set(value);
            let payload = MarginalOptimizer { threshold: value };
            spawn_local(async move {
                let _ = gloo_net::http::Request::post("/api/marginal_optimizer")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&payload).unwrap())
                    .unwrap()
                    .send()
                    .await;
            });
        })
    };
    html! {
        <div class="slider-marginal-optimizer" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;">
            <label style="color:#00ff99;font-weight:bold;">{ "Marginal Optimizer Threshold (%): " }
                <span style="margin-left:1rem;color:#fff;">{ format!("{:.2}", *threshold) }</span>
            </label>
            if *loading {
                <div style="color:#00ff99;opacity:0.8;text-align:center;padding:1rem 0;">{"Loading..."}</div>
            } else {
                <input type="range" min="0.5" max="10.0" step="0.1" value={threshold.to_string()} oninput={oninput} style="width:100%;accent-color:#00ff99;" />
            }
        </div>
    }
}
