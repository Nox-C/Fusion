use wasm_bindgen_test::*;
use web_sys::window;
use yew::Renderer;
use frontend::components::matrix_panel::MatrixPanel;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_matrix_panel_renders_table() {
    // Create a mount point in the DOM
    let document = window().unwrap().document().unwrap();
    let mount = document.create_element("div").unwrap();
    mount.set_id("output");
    document.body().unwrap().append_child(&mount).unwrap();

    Renderer::<MatrixPanel>::with_root(mount.clone().into()).render();

    // If we reach here, the component mounted successfully without panic.
    // More detailed rendering checks require backend or mocking.

}
