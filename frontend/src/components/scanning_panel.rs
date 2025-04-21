use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct ScanInfo {
    pub id: String,
    pub pair: String,
    pub chain: String,
    pub opportunity: f64,
    pub status: String,
}

#[function_component(ScanningPanel)]
pub fn scanning_panel() -> Html {
    let scanning = use_state(|| Vec::<ScanInfo>::new());
    let loading = use_state(|| true);
    {
        let scanning = scanning.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local({
                let scanning = scanning.clone();
                let loading = loading.clone();
                async move {
                    let resp = gloo_net::http::Request::get("/api/scanning")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<Vec<ScanInfo>>().await {
                            scanning.set(data);
                        }
                    }
                    loading.set(false);
                }
            });
            || ()
        });
    }
    html! {
        <div class="scanning-panel" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;overflow-x:auto;">
            <h3 style="color:#00ff99;">{ "Live Arbitrage Scanning" }</h3>
            if *loading {
                <div style="color:#00ff99;opacity:0.8;text-align:center;padding:1.5rem 0;">{"Loading..."}</div>
            } else if scanning.is_empty() {
                <p style="color:#bbb;margin-top:1rem;">{"No arbitrage opportunities detected."}</p>
            } else {
                <table style="width:100%;margin-top:1rem;color:#fff;font-size:1rem;border-collapse:collapse;">
                    <thead>
                        <tr style="background:#20242b;">
                            <th style="padding:0.5rem 1rem;">{"ID"}</th>
                            <th style="padding:0.5rem 1rem;">{"Pair"}</th>
                            <th style="padding:0.5rem 1rem;">{"Chain"}</th>
                            <th style="padding:0.5rem 1rem;">{"Opportunity"}</th>
                            <th style="padding:0.5rem 1rem;">{"Status"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for scanning.iter().map(|info| html! {
                            <tr style="border-bottom:1px solid #333;transition:background 0.2s;" onmouseover={Callback::from(|e: MouseEvent| {
                                let tr = e.target_dyn_into::<web_sys::Element>().unwrap().closest("tr").unwrap().unwrap();
                                tr.set_attribute("style", "background:#222831;border-bottom:1px solid #00ff99;transition:background 0.2s;").ok();
                            })} onmouseout={Callback::from(|e: MouseEvent| {
                                let tr = e.target_dyn_into::<web_sys::Element>().unwrap().closest("tr").unwrap().unwrap();
                                tr.set_attribute("style", "border-bottom:1px solid #333;transition:background 0.2s;").ok();
                            })}>
                                <td style="padding:0.5rem 1rem;">{ &info.id }</td>
                                <td style="padding:0.5rem 1rem;">{ &info.pair }</td>
                                <td style="padding:0.5rem 1rem;">{ &info.chain }</td>
                                <td style="padding:0.5rem 1rem;">{ format!("{:.2}%", info.opportunity) }</td>
                                <td style="padding:0.5rem 1rem;">{ &info.status }</td>
                            </tr>
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
