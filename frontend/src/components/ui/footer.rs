use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FooterProps {
    #[prop_or_else(|| "1.0.0".to_string())]
    pub version: String,
    #[prop_or_default]
    pub dex_count: usize,
    #[prop_or(true)]
    pub is_online: bool,
}

#[function_component(Footer)]
pub fn footer(props: &FooterProps) -> Html {
    html! {
        <footer class="bg-[hsl(var(--card))] border-t border-gray-800 p-4 mt-auto">
            <div class="container mx-auto flex justify-between items-center">
                <span class="text-sm text-muted-foreground">{ format!("FUSION © {}", props.version) }</span>
                <span class="text-sm text-muted-foreground">{ format!("DEXs Monitored: {}", props.dex_count) }</span>
                <span class={ if props.is_online { "text-[hsl(var(--success))]" } else { "text-destructive" } }>
                    { if props.is_online { "Online" } else { "Offline" } }
                </span>
            </div>
        </footer>
    }
}
