use wasm_bindgen::prelude::*;
use web_sys::Element;

#[wasm_bindgen(module = "@material/drawer")]
extern "C" {
    #[derive(Debug, Clone)]
    pub(crate) type MDCDrawer;

    #[wasm_bindgen(static_method_of = MDCDrawer, catch)]
    pub(crate) fn attachTo(element: &Element) -> Result<MDCDrawer, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub(crate) fn open(this: &MDCDrawer) -> bool;

    #[wasm_bindgen(method, setter)]
    pub(crate) fn set_open(this: &MDCDrawer, val: bool);
}
