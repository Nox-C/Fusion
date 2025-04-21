// Yew component for dashboard footer (scaffold)
use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer class="dashboard-footer">
            <p>{ "Fusion © 2025" }</p>
        </footer>
    }
}
