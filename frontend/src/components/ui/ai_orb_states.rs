use yew::{function_component, html, Html, Properties};

#[derive(PartialEq, Clone, Copy)]
pub enum OrbState {
    Idle,
    Listening,
    Speaking,
    Thinking,
    Error,
}

#[derive(Properties, PartialEq)]
pub struct AIOrbStatesProps {
    pub state: OrbState,
    pub message: String,
}

#[function_component(AIOrbStates)]
pub fn ai_orb_states(props: &AIOrbStatesProps) -> Html {
    let (pulse, icon) = match props.state {
        OrbState::Speaking => ("ai-orb-pulse-speaking", "\u{1F5E3}"), // 🗣️
        OrbState::Listening => ("ai-orb-pulse-listening", "\u{1F50A}"), // 🔊
        OrbState::Thinking => ("ai-orb-pulse-thinking", "\u{1F914}"), // 🤔
        OrbState::Error => ("ai-orb-pulse-error", "\u{26A0}"), // ⚠️
        OrbState::Idle => ("ai-orb-pulse-idle", "\u{1F311}"), // 🌑
    };
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
                {if matches!(props.state, OrbState::Speaking | OrbState::Listening) {
                    html! {
                        <image
                            href="/ai_face_small.png"
                            x="20" y="18" width="56" height="56"
                            style="opacity: 0.72; transition: opacity 0.8s cubic-bezier(.4,2,.6,1); mix-blend-mode: lighten; filter: drop-shadow(0 0 8px #00fff7cc);"
                        />
                    }
                } else {
                    html! {}
                }}
                <text x="48" y="90" text-anchor="middle" font-size="24" fill="#ae00ff" opacity="0.7">{icon}</text>
            </svg>
            <div class="ai-orb-message">
                { &props.message }
            </div>
        </div>
    }
}
