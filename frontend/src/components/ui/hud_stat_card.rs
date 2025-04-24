use yew::{function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct HudStatCardProps {
    pub label: String,
    pub value: String,
    pub percent: f64,
    pub color: String, // neon color
}

#[function_component(HudStatCard)]
pub fn hud_stat_card(props: &HudStatCardProps) -> Html {
    html! {
        <div class="hud-stat-card">
            <svg width="120" height="120" viewBox="0 0 120 120">
                <defs>
                    <linearGradient id="statGlow" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0%" stop-color="#00fff7" />
                        <stop offset="100%" stop-color="#ae00ff" />
                    </linearGradient>
                </defs>
                <circle cx="60" cy="60" r="50" stroke="#222" stroke-width="8" fill="none" />
                <circle 
                    cx="60" cy="60" r="50" 
                    stroke={props.color.clone()} 
                    stroke-width="8" 
                    fill="none" 
                    stroke-dasharray="314" 
                    stroke-dashoffset={format!("{}", 314.0 - 314.0 * props.percent)}
                    style="filter: drop-shadow(0 0 16px #00fff7); transition: stroke-dashoffset 1s;"
                />
                <text x="50%" y="54%" text-anchor="middle" fill="#fff" font-size="1.6em" font-family="Orbitron, monospace" dominant-baseline="middle">{&props.value}</text>
            </svg>
            <div class="hud-stat-label">{&props.label}</div>
        </div>
    }
}
