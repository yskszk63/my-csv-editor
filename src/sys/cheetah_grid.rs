use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use js_sys::Object;

#[wasm_bindgen(module = "cheetah-grid")]
extern "C" {
    #[wasm_bindgen(js_namespace = ["ListGrid", "EVENT_TYPE"])]
    pub(crate) static CHANGED_VALUE: String;

    #[derive(Debug, Clone)]
    pub(crate) type ListGrid;

    #[wasm_bindgen(js_name = "ListGrid", constructor, catch)]
    pub(crate) fn new(options: Option<&Object>) -> Result<ListGrid, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub(crate) fn listen(this: &ListGrid, type_: &str, callback: &Closure<dyn FnMut(Object)>) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch)]
    pub(crate) fn invalidate(this: &ListGrid) -> Result<(), JsValue>;

    #[wasm_bindgen(method, getter, catch, js_name = "dataSource")]
    pub(crate) fn data_source(this: &ListGrid) -> Result<CachedDataSource, JsValue>;

    #[wasm_bindgen(method, getter, catch)]
    pub(crate) fn selection(this: &ListGrid) -> Result<Object, JsValue>;

    pub(crate) type InlineInputEditor;

    #[wasm_bindgen(constructor, catch, js_namespace = ["columns", "action"])]
    pub(crate) fn new() -> Result<InlineInputEditor, JsValue>;

    pub(crate) type CachedDataSource;

    #[wasm_bindgen(constructor, catch, js_namespace = ["data"])]
    pub(crate) fn new(opt: Object) -> Result<CachedDataSource, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "clearCache")]
    pub(crate) fn clear_cache(this: &CachedDataSource) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub(crate) fn length(this: &CachedDataSource) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, setter)]
    pub(crate) fn set_length(this: &CachedDataSource, val: usize) -> Result<(), JsValue>;

}
