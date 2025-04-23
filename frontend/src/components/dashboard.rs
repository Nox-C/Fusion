// Yew component for the dashboard (scaffold)
use yew::prelude::*;
use crate::components::matrix_panel::MatrixPanel;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    html! {
        <div class="dashboard" style="min-height:100vh;background:#181c20;padding:0;margin:0;display:flex;flex-direction:column;align-items:center;">
            <crate::components::header::Header />
            <MatrixPanel />
            <crate::components::footer::Footer />
        </div>
    }
}
