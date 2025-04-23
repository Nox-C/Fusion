// Yew component for dashboard header (scaffold)
use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="dashboard-header">
            <h1 style="font-family: 'Montserrat', 'Segoe UI', 'Arial', sans-serif; color: #f8faff; font-size: 2.5rem; letter-spacing: 0.04em; text-shadow: 0 2px 24px #fff8, 0 1px 2px #0003; margin-bottom: 0.5rem;">{"Fusion"}</h1>
        </header>
    }
}
