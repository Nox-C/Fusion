use yew::{function_component, html, Html};
use crate::components::ui::hud_stat_card::HudStatCard;
use crate::api::profit::fetch_profit;
use yew::use_effect;
use wasm_bindgen_futures::spawn_local;
use crate::components::ui::ai_orb::AIOrb;
use crate::components::ui::ai_orb_states::{AIOrbStates, OrbState};
use crate::components::ui::fusion_chat::FusionChat;

/// Floating AI orb with animation and message
fn ai_orb_panel(state: OrbState, message: &str) -> Html {
    html! {
        <div style="position:fixed;bottom:2.5rem;right:2.5rem;z-index:1000;">
            <AIOrbStates state={state} message={message.to_string()} />
        </div>
    }
}

// TODO: Only one Dashboard function_component should exist. This is the canonical version.
#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    use yew::use_state;
    use yew::use_effect;
    use std::rc::Rc;
    use gloo_timers::callback::Interval;

    let orb_states = vec![
        (OrbState::Idle, "Fusion is idle, awaiting input..."),
        (OrbState::Listening, "Listening for your command..."),
        (OrbState::Thinking, "Analyzing arbitrage opportunities..."),
        (OrbState::Speaking, "Maximizing arbitrage profits!"),
        (OrbState::Error, "System anomaly detected.")
    ];
    let state_idx = use_state(|| 0);
    let orb_states = Rc::new(orb_states);

    // PROFIT STATE
    enum ProfitState {
        Loading,
        Loaded(f64),
        Error(String),
    }
    let profit_state = use_state(|| ProfitState::Loading);
    {
        let profit_state = profit_state.clone();
        use_effect(move || {
            spawn_local(async move {
                match fetch_profit().await {
                    Some(val) => profit_state.set(ProfitState::Loaded(val)),
                    None => profit_state.set(ProfitState::Error("Failed to fetch profit".into())),
                }
            });
            || ()
        });
    }

    {
        let state_idx = state_idx.clone();
        let orb_states = orb_states.clone();
        use_effect(move || {
            let interval = Interval::new(2000, move || {
                state_idx.set(((*state_idx + 1) % orb_states.len()) as usize);
            });
            move || drop(interval)
        });
    }
    let (orb_state, orb_msg) = orb_states[*state_idx].clone();

    html! {
        <div class="fusion-dashboard">
            { hud_background() }
            { ai_orb_panel(orb_state, orb_msg) }
            <div style="position:fixed;bottom:2.5rem;right:22rem;z-index:1100;max-width:340px;">
                <FusionChat />
            </div>
            <div class="hud-stat-row" style="display:flex;gap:2rem;justify-content:center;margin-top:5rem;">
                {
                    match &*profit_state {
                        ProfitState::Loading => html! { <HudStatCard label="Profit" value="Loading..." percent={0.0} color="#00fff7" /> },
                        ProfitState::Loaded(val) => {
                            let percent = if *val > 0.0 { 1.0 } else { 0.0 };
                            html! { <HudStatCard label="Profit" value={format!("${:.0}", val)} percent={percent} color="#00fff7" /> }
                        },
                        ProfitState::Error(e) => html! { <HudStatCard label="Profit" value={format!("Error: {}", e)} percent={0.0} color="#ff0033" /> },
                    }
                }
                <HudStatCard label="Arbitrages" value="57" percent={0.57} color="#ae00ff" />
                <HudStatCard label="Spread" value="3.2%" percent={0.32} color="#00ffae" />
                <HudStatCard label="Gas Used" value="1.4M" percent={0.44} color="#ff00ae" />
            </div>
            // ... rest of your dashboard content ...
        </div>
    }
}


/// Animated SVG/canvas sci-fi background
fn hud_background() -> Html {
    html! {
        <svg class="hud-bg" width="100%" height="100%" viewBox="0 0 1920 1080" style="position:fixed;top:0;left:0;z-index:0;pointer-events:none">
            <defs>
                <linearGradient id="hudGrid" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#00fff7" stop-opacity="0.13"/>
                    <stop offset="100%" stop-color="#ae00ff" stop-opacity="0.07"/>
                </linearGradient>
            </defs>
            { for (0..=24).map(|i| html!{
                <line x1={format!("{}", i*80)} y1="0" x2={format!("{}", i*80)} y2="1080" stroke="url(#hudGrid)" stroke-width="1" />
            })}
            { for (0..=13).map(|i| html!{
                <line x1="0" y1={format!("{}", i*80)} x2="1920" y2={format!("{}", i*80)} stroke="url(#hudGrid)" stroke-width="1" />
            })}
            <circle cx="1600" cy="200" r="80" stroke="#00fff7" stroke-width="2" fill="none" opacity="0.3">
                <animate attributeName="r" values="80;120;80" dur="4s" repeatCount="indefinite" />
            </circle>
        </svg>
    }
}

/// Sci-fi sidebar for navigation and system status
fn sidebar() -> Html {
    html! {
        <nav class="sidebar glassmorph-neon">
            <div class="sidebar-logo neon-text">{"FUSION"}</div>
            <ul class="sidebar-nav">
                <li>{"OVERVIEW"}</li>
                <li>{"MATRIX"}</li>
                <li>{"LOGS"}</li>
                <li>{"CONTROL"}</li>
                <li>{"AI ASSISTANT"}</li>
            </ul>
            <div class="sidebar-status">
                <span class="status-dot online"></span> {"SYSTEM ONLINE"}
            </div>
        </nav>
    }
}

// (Removed duplicate Dashboard component to fix build errors)

// All legacy/duplicate dashboard HTML/code removed. Only the main Dashboard component remains above.
