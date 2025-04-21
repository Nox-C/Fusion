use wasm_bindgen_test::*;
use web_sys::window;
use yew::Renderer;
use frontend::components::slider_liquidity::SliderLiquidity;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_slider_liquidity_mounts() {
    let document = window().unwrap().document().unwrap();
    let mount = document.create_element("div").unwrap();
    mount.set_id("output");
    document.body().unwrap().append_child(&mount).unwrap();

    Renderer::<SliderLiquidity>::with_root(mount.clone().into()).render();
}
