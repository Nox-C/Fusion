use wasm_bindgen_test::*;
use web_sys::window;
use yew::Renderer;
use frontend::components::wallet_connect::WalletConnect;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wallet_connect_mounts() {
    let document = window().unwrap().document().unwrap();
    let mount = document.create_element("div").unwrap();
    mount.set_id("output");
    document.body().unwrap().append_child(&mount).unwrap();

    Renderer::<WalletConnect>::with_root(mount.clone().into()).render();
}
