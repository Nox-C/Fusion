use yew::prelude::*;
use wasm_bindgen::closure::Closure;


// TODO: To use canvas, enable HtmlCanvasElement and CanvasRenderingContext2d features in web-sys in Cargo.toml
// use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
// use yew_hooks::use_interval; // Removed unresolved import
// use gloo::timers::future::TimeoutFuture; // Remove unresolved import

use gloo_timers::future::TimeoutFuture;
use rand::Rng;
use wasm_bindgen_futures::spawn_local;

// Define a macro for setting styles dynamically
macro_rules! set_style {
    ($element:ident, $($name:literal: $value:expr,)*) => {
        {
            let mut style_string = String::new();
            $(
                style_string.push_str(&format!("{}:{};", $name, $value));
            )*
            $element.set_attribute("style", &style_string).unwrap();
        }
    };
}

// Custom hook for animated counter (Yew version)
#[hook]
fn use_counter_animation(target_value: i32, duration: u64, delay: u64) -> i32 {
    let value = use_state(|| 0);
    let start_time = use_state(|| 0.0);
    let animation_frame = use_state(|| None::<Closure<dyn FnMut()>>);
    let is_animating = use_state(|| false);
    let target = target_value;
    let val = (*value).clone();
    let set_val = value.setter();

    let animate = {
        let start = start_time.clone();
        let frame = animation_frame.clone();
        let animating = is_animating.clone();
        Callback::from(move |_| {
            if !*animating {
                start.set(/* TODO: performance.now() logic (gloo not available) */ 0.0);
                animating.set(true);

                let update_value = {
                    let start_t = start.clone();
                    let val_setter = set_val.clone();
                    let duration_ms = duration as f64;
                    let target_v = target;
                    let anim_frame = frame.clone();
                    let animating_state = animating.clone();
                    Closure::<dyn FnMut()>::new(move || {
                        let now = /* TODO: performance.now() logic (gloo not available) */ 0.0;
                        let elapsed = now - *start_t; // start_t is UseStateHandle<f64>
                        if elapsed < duration_ms {
                            let next_value = ((elapsed / duration_ms) * target_v as f64) as i32;
                            val_setter.set(next_value);
                            // TODO: request_animation_frame logic not implemented in this context.
                            let handle = None;
                            anim_frame.set(handle);
                        } else {
                            val_setter.set(target_v);
                            animating_state.set(false);
                            if let Some(handle) = (*anim_frame).as_ref() {
                                // TODO: cancel_animation_frame not implemented in this context.
                                anim_frame.set(None);
                            }
                        }
                    })
                    
                };

                // TODO: request_animation_frame not implemented in this context.
                let handle = None;
                frame.set(handle);
            }
        })
    };

    {
        let animate_cb = animate.clone();
        use_effect_with((), move |_| {
            let timeout = TimeoutFuture::new(delay as u32);
            spawn_local(async move {
                timeout.await;
                animate_cb.emit(());
            });
            || {}
        });
    }

    val
}

// Progress bar with animation (Yew version)
#[derive(Properties, PartialEq)]
pub struct AnimatedProgressBarProps {
    pub value: i32,
    pub color: String,
}

#[function_component(AnimatedProgressBar)]
fn animated_progress_bar(props: &AnimatedProgressBarProps) -> Html {
    let width = use_state(|| 0);
    let value = props.value;
    let set_width = width.setter();

    {
        use_effect_with((), move |_| {
            let timeout = TimeoutFuture::new(300);
            spawn_local(async move {
                timeout.await;
                set_width.set(value);
            });
            || {}
        });
    }

    html! {
        <div class="animated-progress-bar">
            <div
                class="bar"
                style={format!("width: {}%; background-color: {}; box-shadow: 0 0 8px {};", *width, props.color, props.color)}
            >
            </div>
        </div>
    }
}

// Pulse effect component (Yew version)
#[function_component(PulseEffect)]
fn pulse_effect() -> Html {
    html! {
        <span class="absolute w-full h-full bg-cyan-500 rounded-full animate-ping opacity-30"></span>
    }
}

#[derive(Properties, PartialEq)]
pub struct StatusIndicatorProps {
    pub status: String,
}

#[function_component(StatusIndicator)]
fn status_indicator(props: &StatusIndicatorProps) -> Html {
    let status_colors = match props.status.as_str() {
        "normal" => "bg-emerald-500",
        "warning" => "bg-amber-500",
        "critical" => "bg-red-500",
        _ => "bg-gray-500",
    };

    html! {
        <div class="status-indicator">
            <span class={format!("status-dot {}", props.status)}></span>
            {
                if props.status != "normal" {
                    html! { <PulseEffect /> }
                } else {
                    Html::default()
                }
            }
        </div>
    }
}

// Stat card component (Yew version)
#[derive(Properties, PartialEq)]
pub struct StatCardProps {
    pub icon: AttrValue,
    pub title: String,
    pub value: i32,
    pub status: String,
    pub color: String,
}

#[function_component(StatCard)]
fn stat_card(props: &StatCardProps) -> Html {
    let animated_value = use_counter_animation(props.value, 2000, 0);
    let icon = props.icon.clone();

    html! {
        <div class="stat-card">
            <div class="absolute top-0 right-0 w-36 h-36 bg-gradient-to-br from-blue-600 to-transparent opacity-25 rounded-full transform translate-x-10 -translate-y-10 blur-md"></div>
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center space-x-3">
                    <div class={format!("p-3 {} bg-opacity-30 rounded-lg", props.color)}>
                        /* TODO: LucideIcon component not found. Icon display disabled. */
                    </div>
                    <h3>{props.title.clone()}</h3>
                </div>
                <StatusIndicator status={props.status.clone()} />
            </div>
            <div class="value">{animated_value.to_string()}</div>
            <AnimatedProgressBar value={std::cmp::min(props.value, 100)} color={props.color.clone()} />
        </div>
    }
}

// Terminal output component (Yew version)
#[function_component(TerminalOutput)]
fn terminal_output() -> Html {
    let lines = use_state(|| Vec::<TerminalLine>::new());
    let set_lines = lines.setter();
    let entries = vec![
        TerminalLine { text: "System initialized".to_string(), type_: "info".to_string() },
        TerminalLine { text: "Network connections established".to_string(), type_: "success".to_string() },
        TerminalLine { text: "Scanning for vulnerabilities...".to_string(), type_: "info".to_string() },
        TerminalLine { text: "Warning: Unauthorized access attempt blocked".to_string(), type_: "warning".to_string() },
        TerminalLine { text: "Memory optimization complete: 16% improvement".to_string(), type_: "success".to_string() },
        TerminalLine { text: "AI systems online - all parameters nominal".to_string(), type_: "info".to_string() },
        TerminalLine { text: "Critical: Power fluctuation detected in sector 7".to_string(), type_: "error".to_string() },
        TerminalLine { text: "Deploying countermeasures...".to_string(), type_: "info".to_string() },
        TerminalLine { text: "Threat neutralized".to_string(), type_: "success".to_string() },
    ];
    let set_lines = lines.setter();

    #[derive(Clone, PartialEq)]
    struct TerminalLine {
        text: String,
        type_: String,
    }

    let get_line_color = |type_: &str| -> &str {
        match type_ {
            "success" => "text-emerald-400",
            "warning" => "text-amber-400",
            "error" => "text-red-400",
            _ => "text-blue-400",
        }
    };

    {
        let set_lines = set_lines.clone();
        let entries = entries.clone();
        use_effect_with((), move |_| {
            // For demonstration, add the first entry
            let mut new_lines = Vec::new();
if let Some(entry) = entries.get(0) {
    new_lines.push(entry.clone());
}
set_lines.set(new_lines);
            || {}
        });
    }

    html! {
        <div class="terminal-output">
            {
                (*lines).iter().map(|line| html! {
                    <div class="mb-1 flex">
                        <span class={format!("{} mr-2", get_line_color(&line.type_))}>{">"}</span>
                        <span class="text-zinc-400">{&line.text}</span>
                    </div>
                }).collect::<Html>()
            }
            <div class="animate-pulse">{">"} <span class="inline-block w-2 h-4 bg-cyan-500 ml-1"></span></div>
        </div>
    }
}

// Circular gauge component (Yew version)
#[derive(Properties, PartialEq)]
pub struct CircularGaugeProps {
    pub value: i32,
    pub max_value: i32,
    pub title: String,
    pub color: String,
    pub icon: AttrValue,
}

#[function_component(CircularGauge)]
fn circular_gauge(props: &CircularGaugeProps) -> Html {
    let percentage = (props.value as f64 / props.max_value as f64) * 100.0;
    let animated_value = use_counter_animation(props.value, 1500, 200);
    let circumference = 2.0 * std::f64::consts::PI * 40.0;
    let stroke_dashoffset = circumference - (percentage / 100.0) * circumference;
    let offset = use_state(|| circumference);
    let set_offset = offset.setter();
    let gauge_color = props.color.clone().replace("bg-", "stroke-");
    let icon = props.icon.clone();

    {
        let dash_offset = stroke_dashoffset;
        use_effect_with((), move |_| {
            let timeout = TimeoutFuture::new(500);
            spawn_local(async move {
                timeout.await;
                set_offset.set(dash_offset);
            });
            || {}
        });
    }

    html! {
        <div class="flex flex-col items-center justify-center p-3">
            <div class="relative w-28 h-28 flex items-center justify-center">
                <svg class="w-28 h-28 transform -rotate-90" viewBox="0 0 100 100">
                    <circle
                        cx="50" cy="50" r="40"
                        stroke="#374151" stroke-width="8" fill="transparent"
                    />
                    <circle
                        cx="50" cy="50" r="40"
                        stroke={gauge_color} stroke-width="8" fill="transparent"
                        stroke-dasharray={circumference.to_string()}
                        stroke-dashoffset={(*offset).to_string()}
                        stroke-linecap="round"
                        class="transition-all duration-1000 ease-out"
                        style="box-shadow: 0 0 8px 1px rgba(0,0,0,0.4);"
                    />
                </svg>
                <div class="absolute inset-0 flex items-center justify-center flex-col">
                    /* TODO: LucideIcon component not found. Icon display disabled. */
                    <span class="text-white text-lg font-bold">{animated_value.to_string()}</span>
                </div>
            </div>
            <div class="text-xs text-zinc-500 mt-2 text-center uppercase tracking-wider">{props.title.clone()}</div>
        </div>
    }
}

// AI Holographic Orb component (Yew version - Enhanced Detail and Pop)
#[function_component(AIHolographicOrb)]
pub fn ai_holographic_orb() -> Html {
    html! { <div class="orb-placeholder">{"Orb Placeholder"}</div> }
}

use crate::components::ui::header::Header;
use crate::components::ui::footer::Footer;
use crate::components::matrix_panel::MatrixPanel;
use crate::components::scanning_panel::ScanningPanel;
use crate::components::slider_liquidity::SliderLiquidity;
use crate::components::slider_marginal_optimizer::SliderMarginalOptimizer;
use crate::components::ui::wallet_connect::WalletConnect;
use crate::components::ui::ai_orb::AIOrb;

// Dashboard component (Yew version)
#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    html! {
        <div class="fusion-dashboard" style="position:relative;min-height:100vh;">
            <Header />

            // 3D Floating Orb (AI) in the center of the dashboard grid with holographic stand
            <div class="dashboard-grid" style="position:relative;">
                <div style="grid-column: 1 / -1; display: flex; flex-direction: column; align-items: center; justify-content: center; margin-bottom: 2.5rem; margin-top: 0.5rem;">
                    <div id="orb3d-canvas" style="width:220px;height:220px;pointer-events:none;z-index:60;"></div>
                    <div class="holo-stand-container" style="margin-top:-32px;z-index:59;">
                        <div class="holo-stand"></div>
                    </div>
                    <div style="display:flex;gap:2.2rem;margin-top:1.2rem;">
                        <canvas id="gauge-1" width="110" height="110" style="background:none;"></canvas>
                        <canvas id="gauge-2" width="110" height="110" style="background:none;"></canvas>
                        <canvas id="gauge-3" width="110" height="110" style="background:none;"></canvas>
                    </div>
                </div>
                <div style="grid-column: 1 / -1;">
                    <div class="glass-panel" style="min-height:320px;max-height:420px;overflow:auto;padding:0.5rem 0.5rem 0 0.5rem;">
                        <div style="color:#00fff7;text-align:left;margin:0 0 0.7rem 0;font-size:1.2rem;font-weight:bold;">{"Live Transaction Analysis"}</div>
                        <div id="tx-window"></div>
                    </div>
                </div>
                <MatrixPanel />
                <ScanningPanel />
                <SliderLiquidity />
                <SliderMarginalOptimizer />
                <WalletConnect />
                <TerminalOutput />
            </div>

            <Footer />
        
            <canvas id="liquid-orb-canvas" width="1920" height="1080" style="position:fixed;top:0;left:0;width:100vw;height:100vh;z-index:10;"></canvas>
            <div class="z-30 absolute left-1/2 top-2/3 -translate-x-1/2 -translate-y-1/2">
                <div class="morphic-message" style="background:rgba(24,28,32,0.97);border:2px solid #00fff7;box-shadow:0 4px 32px #00fff7aa;">
                    {"Hello, Commander. All systems nominal."}
                </div>
            </div>
            <script src="/static/liquid_orb.js"></script>
            <script type="module" src="/static/orb3d.js"></script>
            <script type="module" src="/static/gauges.js"></script>
            <script type="module" src="/static/transaction_window.js"></script>
        </div>
    }
}