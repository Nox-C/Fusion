use yew::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const MOBILE_BREAKPOINT: i32 = 768;

#[hook]
pub fn use_is_mobile() -> bool {
    let window = web_sys::window().expect("No window available");
    let media_query = format!("(max-width: {}px)", MOBILE_BREAKPOINT - 1);
    let matches = window.match_media(&media_query).unwrap().unwrap().matches();
    let is_mobile = use_state(|| matches);

    {
        let is_mobile = is_mobile.clone();
        use_effect_with((), move |_| {
            let mql = window.match_media(&media_query).unwrap().unwrap();
            let closure = Closure::wrap(Box::new(move |_: web_sys::MediaQueryListEvent| {
                let new_match = window.inner_width().unwrap().as_f64().unwrap() < MOBILE_BREAKPOINT as f64;
                is_mobile.set(new_match);
            }) as Box<dyn FnMut(_)>);
            mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).unwrap();
            closure.forget();
            || ()
        });
    }

    *is_mobile
}
