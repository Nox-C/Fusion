// Yew component for displaying matrices (scaffold)
use yew::prelude::*;

use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use gloo_net::websocket::{futures::WebSocket, Message};
use futures_util::StreamExt;
use serde_json;

use std::collections::HashMap;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct DexPrice {
    pub dex: String,
    pub price: f64,
    pub timestamp: u64,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct ArbitrageOpportunity {
    pub matrix_id: String,
    pub buy_dex: String,
    pub sell_dex: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub marginal_optimizer_pct: f64,
    pub chain: String,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct Transaction {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub status: String,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct Matrix {
    pub id: String,
    pub name: String,
    pub chain: String,
    pub marginal_optimizer: f64,
    pub dex_prices: HashMap<String, DexPrice>,
    pub opportunities: Vec<ArbitrageOpportunity>,
    pub recent_transactions: Vec<Transaction>,
    pub status: String,
}

#[function_component(MatrixPanel)]
pub fn matrix_panel() -> Html {
    let matrices = use_state(|| Vec::<Matrix>::new());
    {
        let matrices = matrices.clone();
        use_effect_with((), move |_| {
            // Initial REST fetch
            spawn_local({
                let matrices = matrices.clone();
                async move {
                    let resp = gloo_net::http::Request::get("/api/matrices")
                        .send()
                        .await;
                    if let Ok(resp) = resp {
                        if let Ok(data) = resp.json::<Vec<Matrix>>().await {
                            matrices.set(data);
                        }
                    }
                }
            });
            // WebSocket for live updates
            spawn_local({
                let matrices = matrices.clone();
                async move {
                    let ws = WebSocket::open("ws://localhost:8000/ws/matrices");
                    if let Ok(mut ws) = ws {
                        while let Some(msg) = ws.next().await {
                            if let Ok(Message::Text(txt)) = msg {
                                if let Ok(update) = serde_json::from_str::<Vec<Matrix>>(&txt) {
                                    matrices.set(update);
                                }
                            }
                        }
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="matrix-panel" style="display: flex; flex-wrap: wrap; gap: 2.5rem; background: #181c20; padding: 2rem 0; overflow-x: auto; justify-content: center;">
            <h2 style="width: 100%; font-family: 'Montserrat', 'Segoe UI', 'Arial', sans-serif; color: #f8faff; font-size: 2rem; margin-bottom: 1.5rem; letter-spacing: 0.03em; text-align:center;">{"Matrices"}</h2>
            { for matrices.iter().map(|m| html! {
                <div class="matrix-card" style="background: rgba(248,250,255,0.95); border-radius: 20px; box-shadow: 0 6px 32px #00ff9955, 0 1.5px 8px #0002; padding: 2rem 1.5rem; min-width: 320px; max-width: 420px; flex: 1 1 340px; margin-bottom: 1rem; transition: box-shadow 0.2s, transform 0.2s; font-family: 'Montserrat', 'Segoe UI', 'Arial', sans-serif; border: 1.5px solid #00ff99; position: relative; overflow: hidden; cursor: pointer; outline: none; backdrop-filter: blur(8px); will-change: box-shadow, transform; }
                    &:hover { box-shadow: 0 8px 40px #00ff9977, 0 2px 12px #0003; transform: translateY(-3px) scale(1.015); }">
                    <h3 style="margin-top: 0; color: #23272f; font-size: 1.35rem; font-weight: 700; letter-spacing: 0.02em; text-shadow: 0 1px 8px #fff6;">{ format!("{} ({})", m.name, m.chain) }</h3>
                    <div style="margin-bottom: 0.5rem; color: #23272f; font-size: 1.02rem;">
                        <b style="color: #888; font-weight: 500;">{"Matrix ID:"}</b> { &m.id }
                    </div>
                    <div style="margin-bottom: 0.5rem; color: #23272f; font-size: 1.02rem;">
                        <b style="color: #888; font-weight: 500;">{"Status:"}</b> { &m.status }
                    </div>
                    <div style="margin-bottom: 0.5rem; color: #23272f; font-size: 1.02rem;">
                        <b style="color: #888; font-weight: 500;">{"Marginal Optimizer:"}</b> <span style="color:#23272f; font-weight:600;">{ format!("{:.4}", m.marginal_optimizer) }</span>
                    </div>
                    <div style="margin-bottom: 0.5rem; color: #23272f;">
                        <b style="color: #888; font-weight: 500;">{"DEX Prices:"}</b>
                        <table style="width: 100%; background: #f8faff; border-radius: 8px; margin-top: 0.25rem; box-shadow: 0 1px 4px #f8faff22;">
                            <thead>
                                <tr>
                                    <th style="text-align:left; color:#23272f;">{"DEX"}</th>
                                    <th style="text-align:right; color:#23272f;">{"Price"}</th>
                                    <th style="text-align:right; color:#23272f;">{"Timestamp"}</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for m.dex_prices.values().map(|dp| html! {
                                    <tr>
                                        <td>{ &dp.dex }</td>
                                        <td style="text-align:right;">{ format!("{:.6}", dp.price) }</td>
                                        <td style="text-align:right;">{ dp.timestamp }</td>
                                    </tr>
                                })}
                            </tbody>
                        </table>
                    </div>
                    <div style="margin-bottom: 0.5rem; color: #23272f;">
                        <b style="color: #888; font-weight: 500;">{"Opportunities:"}</b>
                        <ul style="margin: 0.25rem 0 0 1rem; color: #23272f;">
                            { for m.opportunities.iter().map(|op| html! {
                                <li>
                                    <span style="font-weight:600;">{format!("{} → {}", op.buy_dex, op.sell_dex)}</span>
                                    {format!(" | Buy: {:.6} | Sell: {:.6} | %: {:.2} | Chain: {}", op.buy_price, op.sell_price, op.marginal_optimizer_pct, op.chain)}
                                </li>
                            }) }
                        </ul>
                    </div>
                    <div style="margin-bottom: 0.5rem; color: #23272f;">
                        <b style="color: #888; font-weight: 500;">{"Recent Transactions:"}</b>
                        <ul style="margin: 0.25rem 0 0 1rem; color: #23272f;">
                            { for m.recent_transactions.iter().map(|tx| html! {
                                <li>
                                    <span style="font-weight:600;">{&tx.tx_hash[..8]}</span>
                                    {format!(" | From: {} | To: {} | Amount: {:.4} | Status: {} | Time: {}", tx.from, tx.to, tx.amount, tx.status, tx.timestamp)}
                                </li>
                            }) }
                        </ul>
                    </div>
                </div>
            })}
        </div>
    }
}
