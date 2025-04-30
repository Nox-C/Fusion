use yew::{function_component, html, Html, Properties, Callback, use_effect_with, use_state};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Properties, PartialEq)]
pub struct AIOrbProps {
    pub speaking: bool,
    pub listening: bool,
    pub message: String,
}

#[function_component(AIOrb)]
pub fn ai_orb(props: &AIOrbProps) -> Html {
    let listening = use_state(|| props.listening);
    let speaking = use_state(|| props.speaking);
    let message = use_state(|| props.message.clone());

    // Bridge to JS voice handler
    use_effect_with((), {
        let listening = listening.clone();
        let speaking = speaking.clone();
        let message = message.clone();
        move |_| {
            let cb_listen = Closure::wrap(Box::new(move |val: bool| {
                listening.set(val);
            }) as Box<dyn FnMut(bool)>);
            let cb_speak = Closure::wrap(Box::new(move |val: bool| {
                speaking.set(val);
            }) as Box<dyn FnMut(bool)>);
            let cb_transcript = Closure::wrap(Box::new(move |transcript: JsValue| {
                if let Some(text) = transcript.as_string() {
                    message.set(text);
                }
            }) as Box<dyn FnMut(JsValue)>);
            // Attach to global FusionAIVoice if present
            let win = web_sys::window().unwrap();
            if let Some(voice) = js_sys::Reflect::get(&win, &JsValue::from_str("FusionAIVoice")).ok().and_then(|v| v.dyn_into::<js_sys::Object>().ok()) {
                js_sys::Reflect::set(&voice, &JsValue::from_str("onListening"), cb_listen.as_ref().unchecked_ref()).ok();
                js_sys::Reflect::set(&voice, &JsValue::from_str("onSpeaking"), cb_speak.as_ref().unchecked_ref()).ok();
                js_sys::Reflect::set(&voice, &JsValue::from_str("onTranscript"), cb_transcript.as_ref().unchecked_ref()).ok();
            }
            cb_listen.forget();
            cb_speak.forget();
            cb_transcript.forget();
            || ()
        }
    });

    let on_microphone = {
        Callback::from(move |_| {
            let win = web_sys::window().unwrap();
            if let Some(voice) = js_sys::Reflect::get(&win, &JsValue::from_str("FusionAIVoice")).ok().and_then(|v| v.dyn_into::<js_sys::Object>().ok()) {
                if let Ok(f) = js_sys::Reflect::get(&voice, &JsValue::from_str("startListening")) {
                    if let Some(f) = f.dyn_ref::<js_sys::Function>() {
                        let _ = f.call0(&voice);
                    }
                }
            }
        })
    };

    let pulse = if *speaking { "ai-orb-pulse-speaking" } else if *listening { "ai-orb-pulse-listening" } else { "ai-orb-pulse-idle" };
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
                <g>
                    <circle cx="48" cy="80" r="12" fill="#00fff7" opacity={if *listening { "0.5" } else { "0.08" }}/>
                    <text x="48" y="85" text-anchor="middle" fill="#181c20" font-size="1.6em" font-family="monospace" opacity={if *listening { "0.85" } else { "0.22" }}>{"🎤"}</text>
                </g>
            </svg>
            <button class="ai-orb-mic-btn" onclick={on_microphone} title="Speak to Fusion" style="background:rgba(24,28,32,0.92);border:2px solid #00fff7;border-radius:50%;width:48px;height:48px;position:absolute;left:50%;bottom:-32px;transform:translateX(-50%);box-shadow:0 2px 16px #00fff7aa;cursor:pointer;">
                <span style="font-size:2em;">{"🎤"}</span>
            </button>
            <div class="ai-orb-message" style="margin-top:1.5em;">
                { &*message }
            </div>
            <script src="/static/ai_voice.js"></script>
        </div>
    }
}
