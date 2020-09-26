use wasm_bindgen::prelude::*;
use web_sys::Element;

#[wasm_bindgen(module = "@material/form-field")]
extern "C" {
    #[derive(Debug,Clone)]
    pub(crate) type MDCFormField;

    #[wasm_bindgen(constructor, catch)]
    pub(crate) fn new(element: &Element) -> Result<MDCFormField, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub(crate) fn input(this: &MDCFormField) -> JsValue;

    #[wasm_bindgen(method, setter)]
    pub(crate) fn set_input(this: &MDCFormField, input: &JsValue);
}
