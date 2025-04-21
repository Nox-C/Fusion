// Yew component for dashboard header (scaffold)
use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="dashboard-header">
            <h2>{ "Fusion Arbitrage Dashboard" }</h2>
        </header>
    }
}
