// Yew component for the dashboard (scaffold)
use yew::prelude::*;
use crate::components::matrix_panel::MatrixPanel;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    html! {
        <div class="dashboard">
            <h1>{ "Fusion Dashboard" }</h1>
            <MatrixPanel />
        </div>
    }
}
