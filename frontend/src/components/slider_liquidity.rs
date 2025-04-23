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

    let oninput = {
        let percent = percent.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
            let value = input.value().parse::<u32>().unwrap_or(50);
            percent.set(value);
        })
    };
    html! {
        <div class="slider-liquidity" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;">
            <label style="color:#00ff99;font-weight:bold;">{ "Liquidity Usage (%): " }
                <span style="margin-left:1rem;color:#fff;">{ *percent }</span>
            </label>
            <input type="range" min="1" max="100" step="1" value={percent.to_string()} oninput={oninput} style="width:100%;accent-color:#00ff99;" />
            <div style="color:#bbb;margin-top:1rem;font-size:0.98rem;">{"Note: Liquidity is now managed automatically by the backend in production."}</div>
        </div>
    }
}
