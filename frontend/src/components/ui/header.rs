use yew::prelude::*;
use crate::components::ui::button::Button;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    #[prop_or(true)]
    pub is_monitoring: bool,
    #[prop_or_else(|| "$3,142.58".to_string())]
    pub eth_price: String,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let HeaderProps { is_monitoring, eth_price } = props;
    let wallet_connected = use_state(|| false);
    let onclick = {
        let wallet_connected = wallet_connected.clone();
        Callback::from(move |_| wallet_connected.set(true))
    };

    html! {
        <header class="bg-[hsl(var(--card))] border-b border-gray-800 p-4">
            <div class="container mx-auto flex justify-between items-center">
                <div class="flex items-center space-x-2">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-primary" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M12 7a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0V8.414l-4.293 4.293a1 1 0 01-1.414 0L8 10.414l-4.293 4.293a1 1 0 01-1.414-1.414l5-5a1 1 0 011.414 0L11 10.586 14.586 7H12z" clip-rule="evenodd" />
                    </svg>
                    <h1 class="text-xl font-bold">{"FUSION"}</h1>
                </div>
                <div class="flex items-center space-x-4">
                    { if *is_monitoring {
                        html! {
                            <div class="hidden md:flex items-center">
                                <span class="h-2 w-2 bg-[hsl(var(--success))] rounded-full pulse mr-2"></span>
                                <span class="text-muted-foreground text-sm">{"Monitoring Active"}</span>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                    <div class="bg-[hsl(var(--muted))] px-4 py-2 rounded-md">
                        <span class="text-muted-foreground text-sm mr-2">{"ETH:"}</span>
                        <span class="text-foreground">{eth_price}</span>
                    </div>
                    <Button class="bg-primary hover:bg-primary/90 text-white" onclick={onclick}>
                        { if *wallet_connected { "Wallet Connected" } else { "Connect Wallet" } }
                    </Button>
                </div>
            </div>
        </header>
    }
}
