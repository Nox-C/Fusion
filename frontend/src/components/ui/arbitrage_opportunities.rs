use yew::prelude::*;
use crate::components::ui::button::Button;

#[derive(Clone, PartialEq, Properties)]
pub struct ArbitrageOpportunity {
    pub id: u32,
    pub token_symbol: String,
    pub source_dex_name: String,
    pub target_dex_name: String,
    pub source_price: f64,
    pub target_price: f64,
    pub spread_percentage: f64,
    pub estimated_profit_eth: f64,
    #[prop_or(false)]
    pub is_loading: bool,
}

#[derive(Properties, PartialEq)]
pub struct ArbitrageOpportunitiesProps {
    pub opportunities: Vec<ArbitrageOpportunity>,
    #[prop_or_default]
    pub on_execute: Callback<u32>,
}

#[function_component(ArbitrageOpportunities)]
pub fn arbitrage_opportunities(props: &ArbitrageOpportunitiesProps) -> Html {
    html! {
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Token"}</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Source DEX"}</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Target DEX"}</th>
                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">{"Spread %"}</th>
                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">{"Profit ETH"}</th>
                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">{"Action"}</th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    { for props.opportunities.iter().map(|opp| {
                        let id = opp.id;
                        let on_click = {
                            let on_execute = props.on_execute.clone();
                            Callback::from(move |_| on_execute.emit(id))
                        };
                        html! {
                            <tr key={id}>
                                <td class="px-6 py-4 whitespace-nowrap">{ &opp.token_symbol }</td>
                                <td class="px-6 py-4 whitespace-nowrap">{ &opp.source_dex_name }</td>
                                <td class="px-6 py-4 whitespace-nowrap">{ &opp.target_dex_name }</td>
                                <td class="px-6 py-4 whitespace-nowrap text-right">{ format!("{:.2}", opp.spread_percentage) }</td>
                                <td class="px-6 py-4 whitespace-nowrap text-right">{ format!("{:.4}", opp.estimated_profit_eth) }</td>
                                <td class="px-6 py-4 whitespace-nowrap text-right">
                                    <Button class="bg-primary hover:bg-primary/90 text-white px-3 py-1 rounded" onclick={on_click}>
                                        {"Execute"}
                                    </Button>
                                </td>
                            </tr>
                        }
                    }) }
                </tbody>
            </table>
        </div>
    }
}
