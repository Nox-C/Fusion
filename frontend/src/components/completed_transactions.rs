use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
    pub id: String,
    pub matrix: String,
    pub profit: f64,
    pub timestamp: String,
    pub tx_hash: String,
}

#[function_component(CompletedTransactions)]
pub fn completed_transactions() -> Html {
    let transactions = use_state(|| Vec::<Transaction>::new());
    let loading = use_state(|| true);
    {
        let transactions = transactions.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local({
                let transactions = transactions.clone();
                async move {
                    let resp = gloo_net::http::Request::get("/api/completed_transactions")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<Vec<Transaction>>().await {
                            transactions.set(data);
                        }
                    }
                    loading.set(false);
                }
            });
            || ()
        });
    }
    html! {
        <div class="completed-transactions" style="background:#23272f;padding:1.5rem;border-radius:12px;box-shadow:0 2px 12px #00ff9940;margin-bottom:2rem;overflow-x:auto;">
            <h3 style="color:#00ff99;">{ "Completed Transactions" }</h3>
            if *loading {
                <div style="color:#00ff99;opacity:0.8;text-align:center;padding:1.5rem 0;">{"Loading..."}</div>
            } else if transactions.is_empty() {
                <p style="color:#bbb;margin-top:1rem;">{"No completed transactions yet."}</p>
            } else {
                <table style="width:100%;margin-top:1rem;color:#fff;font-size:1rem;border-collapse:collapse;">
                    <thead>
                        <tr style="background:#20242b;">
                            <th style="padding:0.5rem 1rem;">{"ID"}</th>
                            <th style="padding:0.5rem 1rem;">{"Matrix"}</th>
                            <th style="padding:0.5rem 1rem;">{"Profit"}</th>
                            <th style="padding:0.5rem 1rem;">{"Timestamp"}</th>
                            <th style="padding:0.5rem 1rem;">{"Tx Hash"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for transactions.iter().map(|tx| html! {
                            <tr style="border-bottom:1px solid #333;transition:background 0.2s;" onmouseover={Callback::from(|e: MouseEvent| {
                                let tr = e.target_dyn_into::<web_sys::Element>().unwrap().closest("tr").unwrap().unwrap();
                                tr.set_attribute("style", "background:#222831;border-bottom:1px solid #00ff99;transition:background 0.2s;").ok();
                            })} onmouseout={Callback::from(|e: MouseEvent| {
                                let tr = e.target_dyn_into::<web_sys::Element>().unwrap().closest("tr").unwrap().unwrap();
                                tr.set_attribute("style", "border-bottom:1px solid #333;transition:background 0.2s;").ok();
                            })}>
                                <td style="padding:0.5rem 1rem;">{ &tx.id }</td>
                                <td style="padding:0.5rem 1rem;">{ &tx.matrix }</td>
                                <td style="padding:0.5rem 1rem;">{ format!("{:.4}", tx.profit) }</td>
                                <td style="padding:0.5rem 1rem;">{ &tx.timestamp }</td>
                                <td style="padding:0.5rem 1rem;">
                                    <a href={format!("https://etherscan.io/tx/{}", tx.tx_hash)} target="_blank" style="color:#00ff99;text-decoration:underline;">{ &tx.tx_hash[..8] }</a>
                                </td>
                            </tr>
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
