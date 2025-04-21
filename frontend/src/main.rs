use yew::prelude::*;
use frontend::components::{
    dashboard::Dashboard,
    scanning_panel::ScanningPanel,
    completed_transactions::CompletedTransactions,
    wallet_connect::WalletConnect,
    slider_marginal_optimizer::SliderMarginalOptimizer,
    slider_liquidity::SliderLiquidity,
    profit_transfer::ProfitTransfer,
    header::Header,
    footer::Footer,
};

const BG: &str = "#181c20";
const PANEL_BG: &str = "#23272f";
const ACCENT: &str = "#00ff99";
const FONT: &str = "'Segoe UI', 'Arial', sans-serif;";


#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div style={format!("background:{};min-height:100vh;font-family:{}", BG, FONT)}>
            <Header />
            <main style="max-width:1000px;margin:auto;padding:2rem 0;">
                <Dashboard />
                <ScanningPanel />
                <CompletedTransactions />
                <div style="display:flex;gap:2rem;margin-top:2rem;">
                    <WalletConnect />
                    <SliderMarginalOptimizer />
                    <SliderLiquidity />
                    <ProfitTransfer />
                </div>
            </main>
            <Footer />
        </div>
    }
}


fn main() {
    yew::Renderer::<App>::new().render();
}
