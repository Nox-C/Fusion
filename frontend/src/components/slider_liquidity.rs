use yew::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct Liquidity {
    pub percent: u32,
}

#[function_component(SliderLiquidity)]
pub fn slider_liquidity() -> Html {
    let percent = use_state(|| 50u32);
    let loading = use_state(|| true);
    {
        let percent = percent.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local({
                let percent = percent.clone();
                let loading = loading.clone();
                async move {
                    let resp = gloo_net::http::Request::get("/api/liquidity")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<Liquidity>().await {
                            percent.set(data.percent);
                        }
                    }
                    loading.set(false);
                }
            });
            || ()
        });
    }
    let oninput = {
        let percent = percent.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            let value = input.value().parse::<u32>().unwrap_or(50);
            percent.set(value);
            let payload = Liquidity { percent: value };
            spawn_local(async move {
                let _ = gloo_net::http::Request::post("/api/liquidity")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&payload).unwrap())
                    .unwrap()
                    .send()
                    .await;
            });
        })
    };
    html! {
        <div class="slider-liquidity" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;">
            <label style="color:#00ff99;font-weight:bold;">{ "Liquidity Usage (%): " }
                <span style="margin-left:1rem;color:#fff;">{ *percent }</span>
            </label>
            if *loading {
                <div style="color:#00ff99;opacity:0.8;text-align:center;padding:1rem 0;">{"Loading..."}</div>
            } else {
                <input type="range" min="1" max="100" step="1" value={percent.to_string()} oninput={oninput} style="width:100%;accent-color:#00ff99;" />
            }
        </div>
    }
}
