use wasm_bindgen::prelude::*;
use web_sys::Element;

#[wasm_bindgen(module = "@material/top-app-bar")]
extern "C" {
    #[derive(Debug, Clone)]
    pub(crate) type MDCTopAppBar;

    #[wasm_bindgen(constructor, catch)]
    pub(crate) fn new(element: &Element) -> Result<MDCTopAppBar, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "setScrollTarget")]
    pub(crate) fn set_scroll_target(this: &MDCTopAppBar, element: &Element) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch)]
    pub(crate) fn listen(this: &MDCTopAppBar, event: &str, callback: &wasm_bindgen::closure::Closure<dyn FnMut()>) -> Result<(), JsValue>;
}

