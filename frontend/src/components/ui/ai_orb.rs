use yew::{function_component, html, Html, use_state, Callback, Properties};
use gloo_timers::callback::Interval;
use std::rc::Rc;

#[derive(Properties, PartialEq)]
pub struct AIOrbProps {
    pub speaking: bool,
    pub listening: bool,
    pub message: String,
}

#[function_component(AIOrb)]
pub fn ai_orb(props: &AIOrbProps) -> Html {
    let pulse = if props.speaking { "ai-orb-pulse-speaking" } else if props.listening { "ai-orb-pulse-listening" } else { "ai-orb-pulse-idle" };
    html! {
        <div class={format!("ai-orb-container {}", pulse)}>
            <svg width="96" height="96" viewBox="0 0 96 96">
                <defs>
                    <radialGradient id="orbGlow" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stop-color="#00fff7" stop-opacity="1"/>
                        <stop offset="100%" stop-color="#ae00ff" stop-opacity="0.2"/>
                    </radialGradient>
                </defs>
                <circle cx="48" cy="48" r="38" fill="url(#orbGlow)" filter="url(#blur)" opacity="0.8"/>
                <circle cx="48" cy="48" r="28" fill="#181c20" stroke="#00fff7" stroke-width="3"/>
                <circle cx="48" cy="48" r="18" fill="#ae00ff" opacity="0.18"/>
                <circle cx="48" cy="48" r="7" fill="#fff" opacity="0.55"/>
            </svg>
            <div class="ai-orb-message">
                { &props.message }
            </div>
        </div>
    }
}
