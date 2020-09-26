use wasm_bindgen::prelude::*;
use web_sys::Element;

#[wasm_bindgen(module = "@material/menu")]
extern "C" {
    #[derive(Debug, Clone)]
    pub(crate) type MDCMenu;

    #[wasm_bindgen(constructor, catch)]
    pub(crate) fn new(element: &Element) -> Result<MDCMenu, JsValue>;

    #[wasm_bindgen(method, getter)]
    pub(crate) fn open(this: &MDCMenu) -> bool;

    #[wasm_bindgen(method, setter)]
    pub(crate) fn set_open(this: &MDCMenu, val: bool);

    #[wasm_bindgen(method, catch, js_name = "setAbsolutePosition")]
    pub(crate) fn set_absolute_position(this: &MDCMenu, x: i32, y: i32) -> Result<(), JsValue>;
}
