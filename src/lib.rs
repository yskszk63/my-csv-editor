use std::pin::Pin;
use std::task::{Poll, Context};
use std::sync::Arc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{Event, EventTarget, Element, HtmlInputElement, HtmlButtonElement, Url, File, FilePropertyBag, MouseEvent, CustomEvent, HtmlElement};
use js_sys::{Object, Array, Promise, Reflect};
use gloo::events::{EventListener, EventListenerOptions};
use futures::stream::{Stream, StreamExt};
use futures::channel::mpsc::{self, Receiver};
use futures::lock::Mutex;
use csvparser::Csv;

//const DATA: &[u8] = include_bytes!("./c01.csv");

mod cheetah_grid {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::closure::Closure;
    use js_sys::Object;

    #[wasm_bindgen(module = "cheetah-grid")]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["ListGrid", "EVENT_TYPE"])]
        pub(crate) static CHANGED_VALUE: String;

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
}

mod material {
    use wasm_bindgen::prelude::*;
    use web_sys::Element;

    #[wasm_bindgen(module = "@material/top-app-bar")]
    extern "C" {
        pub(crate) type MDCTopAppBar;

        #[wasm_bindgen(constructor, catch)]
        pub(crate) fn new(element: &Element) -> Result<MDCTopAppBar, JsValue>;

        #[wasm_bindgen(method, catch, js_name = "setScrollTarget")]
        pub(crate) fn set_scroll_target(this: &MDCTopAppBar, element: &Element) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub(crate) fn listen(this: &MDCTopAppBar, event: &str, callback: &wasm_bindgen::closure::Closure<dyn FnMut()>) -> Result<(), JsValue>;
    }

    #[wasm_bindgen(module = "@material/form-field")]
    extern "C" {
        pub(crate) type MDCFormField;

        #[wasm_bindgen(constructor, catch)]
        pub(crate) fn new(element: &Element) -> Result<MDCFormField, JsValue>;

        #[wasm_bindgen(method, getter)]
        pub(crate) fn input(this: &MDCFormField) -> JsValue;

        #[wasm_bindgen(method, setter)]
        pub(crate) fn set_input(this: &MDCFormField, input: &JsValue);
    }

    #[wasm_bindgen(module = "@material/checkbox")]
    extern "C" {
        pub(crate) type MDCCheckbox;

        #[wasm_bindgen(constructor, catch)]
        pub(crate) fn new(element: &Element) -> Result<MDCCheckbox, JsValue>;

        #[wasm_bindgen(method, getter)]
        pub(crate) fn checked(this: &MDCCheckbox) -> bool;
    }

    #[wasm_bindgen(module = "@material/list")]
    extern "C" {
        pub(crate) type MDCList;

        #[wasm_bindgen(constructor, catch)]
        pub(crate) fn new(element: &Element) -> Result<MDCList, JsValue>;
    }

    #[wasm_bindgen(module = "@material/drawer")]
    extern "C" {
        #[derive(Clone)]
        pub(crate) type MDCDrawer;

        #[wasm_bindgen(static_method_of = MDCDrawer, catch)]
        pub(crate) fn attachTo(element: &Element) -> Result<MDCDrawer, JsValue>;

        #[wasm_bindgen(method, getter)]
        pub(crate) fn open(this: &MDCDrawer) -> bool;

        #[wasm_bindgen(method, setter)]
        pub(crate) fn set_open(this: &MDCDrawer, val: bool);
    }

    #[wasm_bindgen(module = "@material/menu")]
    extern "C" {
        #[derive(Clone)]
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

}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! js {
    ( $( $key:expr => $val:expr ),* ) => {
        {
            let entries = [
                $( [JsValue::from($key), JsValue::from($val)].iter().collect::<Array>() ),*
            ].iter().collect::<Array>();
            Object::from_entries(&entries).unwrap()
        }
    }
}

struct EventStream<T> {
    queue: Receiver<(T, Event)>,
    _listeners: Vec<EventListener>,
}

impl<T> EventStream<T> where T: Clone + 'static {
    //fn new(target: &EventTarget, events: &[&'static str]) -> Self {
    fn new(desc: &[(&EventTarget, T, &'static str)]) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let mut listeners = vec![];
        for (target, token, event) in desc {
            let token = token.clone();
            let mut tx = tx.clone();
            let opt = EventListenerOptions::enable_prevent_default();
            let listener = EventListener::new_with_options(target, *event, opt, move |event| {
                tx.start_send((token.clone(), event.clone())).unwrap();
            });
            listeners.push(listener);
        }
        Self {
            queue: rx,
            _listeners: listeners,
        }
    }
}

impl<T> Stream for EventStream<T> {
    type Item = (T, Event);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self { ref mut queue, .. } = self.get_mut();
        let polled = Pin::new(queue).poll_next(cx);
        match polled {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some((token, event))) => Poll::Ready(Some((token, event))),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}

struct Grid {
    name: String,

    csv: Arc<Mutex<Csv>>,

    #[allow(dead_code)]
    grid: cheetah_grid::ListGrid,

    #[allow(dead_code)]
    get_record: Closure<dyn FnMut(usize) -> Promise>,

    #[allow(dead_code)]
    on_changed: Closure<dyn FnMut(Object)>,
}

fn load(root: &Element, name: String, data: &str, use_header: bool) -> Result<Grid, JsValue> {
    let document = root.owner_document().unwrap();
    let div = &document.create_element("div").unwrap();
    if let Some(old) = root.first_element_child() {
        old.replace_with_with_node_1(&div).unwrap();
    } else {
        root.append_child(&div).unwrap();
    };

    let inline_input = cheetah_grid::InlineInputEditor::new().unwrap();

    let csv = csvparser::Csv::parse(data, use_header).unwrap();

    let header: Vec<Object> = vec![js! {
        "field" => "n",
        "caption" => "#",
        "sort" => true,
        "width" => "auto",
        "columnType" => "number"
    }].into_iter().chain((0..csv.max_cols()).map(|i| js! {
        "field" => format!("c{}", i),
        "caption" => csv.header(i).unwrap_or(&format!("{}", i)),
        "action" => inline_input.clone(),
        "width" => "auto",
        "sort" => true,
        "columnType" => if csv.cols(i).all(|v| v.parse::<f64>().is_ok()) { "number" } else { "text" }
    })).collect::<Vec<_>>();

    let length = csv.rows() as u32;
    let csv = Arc::new(Mutex::new(csv));

    let get_record = {
        let csv = csv.clone();
        Closure::wrap(Box::new(move |index| {
            let csv = csv.clone();
            future_to_promise(async move {
                let csv = csv.lock().await;
                let row = js! {
                    "n" => format!("{}", index)
                };

                for (i, val) in csv.vals(index).enumerate() {
                    Object::assign(&row, &js! {
                        format!("c{}", i) => val
                    });
                }
                Ok(row.into())
            })
        }) as Box<dyn FnMut(usize) -> Promise>)
    };

    let data_source = cheetah_grid::CachedDataSource::new(js! {
        "get" => get_record.as_ref(),
        "length" => length
    }).unwrap();

    let opt = js! {
        "parentElement" => div,
        "header" => &header.iter().collect::<Array>(),
        "dataSource" => &data_source,
        "font" => "16px monospace",
        "allowRangePaste" => true,
        "keyboardOptions" => &js! {
            "moveCellOnTab" => true,
            "moveCellOnEnter" => true,
            "deleteCellValueOnDel" => true,
            "selectAllOnCtrlA" => true
        }
    };
    let grid = cheetah_grid::ListGrid::new(Some(&opt)).unwrap();

    let on_changed = {
        let csv = csv.clone();
        Closure::wrap(Box::new(move |obj: Object| {
            #[allow(unused_unsafe)]
            let (row, col, value) = unsafe {
                let row = Reflect::get(&obj, &"row".into()).ok();
                let col = Reflect::get(&obj, &"col".into()).ok();
                let value = Reflect::get(&obj, &"value".into()).ok();
                (row, col, value)
            };
            let row = row.as_ref().and_then(JsValue::as_f64).map(|f| f as usize);
            let col = col.as_ref().and_then(JsValue::as_f64).map(|f| f as usize);
            let value = value.as_ref().and_then(JsValue::as_string);

            if let (Some(row), Some(col), Some(val)) = (row, col, value) {
                if row > 0 && col > 0 {
                    let mut csv = csv.try_lock().unwrap();
                    csv.set_val(row - 1, col - 1, val);
                }
            }
        }) as Box<dyn FnMut(Object)>)
    };
    grid.listen(&cheetah_grid::CHANGED_VALUE, &on_changed).unwrap();

    Ok(Grid { get_record, name, csv, grid, on_changed, })
}

#[derive(Debug, Clone)]
enum EventType {
    FileChanged,
    Save,
    ContextMenu,
    MenuSelected,
    AppBarNav,
}

async fn async_main() -> Result<(), JsValue> {
    use EventType::*;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let root = document.query_selector("main").unwrap().unwrap();

    let input_file = document.query_selector("input[type=file]").unwrap().unwrap();
    let app_save = document.query_selector(".app-save").unwrap().unwrap().dyn_into::<HtmlButtonElement>().unwrap();
    let main = document.query_selector("main").unwrap().unwrap();
    let mdc_menu = document.query_selector(".mdc-menu").unwrap().unwrap();
    let header = document.query_selector("header").unwrap().unwrap();

    let top_app_bar = material::MDCTopAppBar::new(&header).unwrap();
    top_app_bar.set_scroll_target(&main).unwrap();
    let drawer = material::MDCDrawer::attachTo(
        &document.query_selector("aside").unwrap().unwrap()).unwrap();
    let formfield = material::MDCFormField::new(
        &document.query_selector(".mdc-form-field").unwrap().unwrap()).unwrap();
    let use_header = material::MDCCheckbox::new(
        &document.query_selector(".mdc-checkbox").unwrap().unwrap()).unwrap();
    formfield.set_input(&use_header);
    let menu = material::MDCMenu::new(&mdc_menu).unwrap();

    let mut grid = None;

    let mut events = EventStream::new(&[
        (input_file.as_ref(), FileChanged, "change"),
        (app_save.as_ref(), Save, "click"),
        (main.as_ref(), ContextMenu, "contextmenu"),
        (mdc_menu.as_ref(), MenuSelected, "MDCMenu:selected"),
        (header.as_ref(), AppBarNav, "MDCTopAppBar:nav"),
    ][..]);

    app_save.set_disabled(true);
    while let Some((token, event)) = events.next().await {
        match token {
            AppBarNav => {
                drawer.set_open(!drawer.open());
            }

            FileChanged => {
                let file_list = event.target()
                    .as_ref().and_then(JsCast::dyn_ref::<HtmlInputElement>)
                    .and_then(HtmlInputElement::files)
                    .map(gloo::file::FileList::from);
                let file_list = if let Some(file_list) = file_list {
                    file_list
                } else {
                    continue
                };

                if let Some(file) = file_list.iter().next() {
                    let bytes = gloo::file::futures::read_as_bytes(file).await.unwrap();

                    let (encoding, _, _) = chardet::detect(&bytes);
                    let coder = encoding::label::encoding_from_whatwg_label(chardet::charset2encoding(&encoding));
                    let text = if let Some(coder) = coder {
                        coder.decode(&bytes, encoding::DecoderTrap::Ignore)
                            .unwrap_or(String::from_utf8_lossy(&bytes).to_string())
                    } else {
                        String::from_utf8_lossy(&bytes).to_string()
                    };
                    grid = Some(load(&root, file.name(), &text, use_header.checked()).unwrap());

                    drawer.set_open(false);
                    app_save.set_disabled(false);
                };
            }

            Save => {
                if let Some(grid) = &grid {
                    let csv = grid.csv.lock().await;
                    let content = format!("{}", &*csv);
                    let parts = Array::of1(&JsValue::from(content));
                    let blob = File::new_with_str_sequence_and_options(
                        &parts,
                        &grid.name,
                        FilePropertyBag::new().type_("text/csv")).unwrap();
                    let url = Url::create_object_url_with_blob(&blob).unwrap();
                    window.location().assign(&url).unwrap();
                    Url::revoke_object_url(&url).unwrap();
                }
            }

            ContextMenu => {
                if grid.is_some() {
                    let event = event.dyn_ref::<MouseEvent>().unwrap();
                    event.prevent_default();
                    menu.set_absolute_position(event.client_x(), event.client_y()).ok();
                    menu.set_open(true);
                }
            }

            MenuSelected => {
                let event = event.dyn_ref::<CustomEvent>().unwrap();
                let detail = event.detail();
                #[allow(unused_unsafe)]
                let item = unsafe {
                    Reflect::get(&detail, &"item".into()).unwrap()
                };
                let item = item.dyn_into::<HtmlElement>().unwrap();
                match (&grid, &item.dataset().get("action")) {
                    (Some(grid), Some(action)) => {
                        let mut csv = grid.csv.lock().await;
                        let grid = &grid.grid;

                        let selection = grid.selection().unwrap();
                        #[allow(unused_unsafe)]
                        let row = unsafe {
                            let select = Reflect::get(&selection, &"select".into()).unwrap();
                            Reflect::get(&select, &"row".into()).unwrap()
                        };
                        let row = row.as_f64().map(|f| f as usize).unwrap();

                        match action.as_ref() {
                            "add_before" => csv.insert_row(row - 1), // FIXME
                            "add_after" => csv.insert_row(row - 0), // FIXME
                            "remove" => csv.remove_row(row - 1), // FIXME
                            _ => {},
                        }
                        let data_source = grid.data_source().unwrap();
                        data_source.set_length(csv.rows()).unwrap();
                        data_source.clear_cache().unwrap();
                        grid.invalidate().unwrap();
                    }
                    _ => {}
                }
            }
        }
    };

    Ok(())
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(Default::default());

    wasm_bindgen_futures::spawn_local(async {
        if let Err(err) = async_main().await {
            log::error!("{:?}", err);
        }
    });
    Ok(())
}
