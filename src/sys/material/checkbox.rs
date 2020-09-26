use wasm_bindgen::prelude::*;
use web_sys::Element;

#[wasm_bindgen(module = "@material/checkbox")]
extern "C" {
    #[derive(Debug, Clone)]
    pub(crate) type MDCCheckbox;

    #[wasm_bindgen(constructor, catch)]
    pub(crate) fn new(element: &Element) -> Result<MDCCheckbox, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub(crate) fn checked(this: &MDCCheckbox) -> bool;
}
