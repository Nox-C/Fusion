use yew::prelude::*;
mod components;
pub mod api;

use self::components::dashboard::Dashboard;
use self::components::ui::header::Header;
use self::components::ui::footer::Footer;
use self::components::ui::wallet_connect::WalletConnect;

const BG: &str = "#181c20";

const FONT: &str = "'Segoe UI', 'Arial', sans-serif;";


#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div style={format!("background:{};min-height:100vh;font-family:{}", BG, FONT)}>
            <Header />
            <main style="max-width:1000px;margin:auto;padding:2rem 0;">
                <Dashboard />
                <WalletConnect />
            </main>
            <Footer />
        </div>
    }
}


fn main() {
    yew::Renderer::<App>::new().render();
}
