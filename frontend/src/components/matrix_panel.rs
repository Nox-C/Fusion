// Yew component for displaying matrices (scaffold)
use yew::prelude::*;

use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use gloo_net::websocket::{futures::WebSocket, Message};
use futures_util::StreamExt;
use serde_json;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct Matrix {
    pub id: String,
    pub name: String,
    pub chain: String,
    pub profit: f64,
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
        <div class="matrix-panel">
            <h2>{ "Matrices" }</h2>
            <table style="width:100%;margin-top:1rem;">
                <thead>
                    <tr>
                        <th>{"ID"}</th>
                        <th>{"Name"}</th>
                        <th>{"Chain"}</th>
                        <th>{"Profit"}</th>
                        <th>{"Status"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for matrices.iter().map(|m| html! {
                        <tr>
                            <td>{ &m.id }</td>
                            <td>{ &m.name }</td>
                            <td>{ &m.chain }</td>
                            <td>{ format!("{:.4}", m.profit) }</td>
                            <td>{ &m.status }</td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </div>
    }
}
