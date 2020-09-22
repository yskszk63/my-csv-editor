use std::pin::Pin;
use std::task::{Poll, Context};
use std::sync::Arc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Event, EventTarget, Element, HtmlInputElement, HtmlButtonElement, Url, Blob, BlobPropertyBag};
use js_sys::{Object, Array};
use gloo::events::EventListener;
use futures::stream::{Stream, StreamExt};
use futures::channel::mpsc::{self, Receiver};
use futures::lock::Mutex;
use csvparser::Csv;

//const DATA: &[u8] = include_bytes!("./c01.csv");

mod cheetah_grid {
    use wasm_bindgen::prelude::*;
    use js_sys::Object;

    #[wasm_bindgen(module = "cheetah-grid")]
    extern "C" {
        pub(crate) type ListGrid;

        #[wasm_bindgen(js_name = "ListGrid", constructor, catch)]
        pub(crate) fn new(options: Option<&Object>) -> Result<ListGrid, JsValue>;

        pub(crate) type InlineInputEditor;

        #[wasm_bindgen(js_name = "ListGrid", constructor, catch, js_namespace = ["columns", "action"])]
        pub(crate) fn new() -> Result<InlineInputEditor, JsValue>;

        pub(crate) type CachedDataSource;

        #[wasm_bindgen(js_name = "ListGrid", constructor, catch, js_namespace = ["data"])]
        pub(crate) fn new(opt: Object) -> Result<CachedDataSource, JsValue>;
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
            let listener = EventListener::new(target, *event, move |event| {
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
    csv: Arc<Mutex<Csv>>,

    #[allow(dead_code)]
    grid: cheetah_grid::ListGrid,

    #[allow(dead_code)]
    get_record: Closure<dyn FnMut(usize) -> Object>
}

fn load(root: &Element, data: &str, use_header: bool) -> Result<Grid, JsValue> {
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
            let csv = csv.try_lock().unwrap();
            let row = js! {
                "n" => format!("{}", index)
            };

            for (i, val) in csv.vals(index).enumerate() {
                Object::assign(&row, &js! {
                    format!("c{}", i) => val
                });
            }
            row
        }) as Box<dyn FnMut(usize) -> Object>)
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

    Ok(Grid { get_record, csv, grid, })
}

#[derive(Debug, Clone)]
enum EventType {
    FileChanged,
    Save,
}

async fn async_main() -> Result<(), JsValue> {
    use EventType::*;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let root = document.query_selector("main").unwrap().unwrap();

    let top_app_bar = material::MDCTopAppBar::new(
        &document.query_selector("header").unwrap().unwrap()).unwrap();
    let drawer = material::MDCDrawer::attachTo(
        &document.query_selector("aside").unwrap().unwrap()).unwrap();
    let formfield = material::MDCFormField::new(
        &document.query_selector(".mdc-form-field").unwrap().unwrap()).unwrap();
    let use_header = material::MDCCheckbox::new(
        &document.query_selector(".mdc-checkbox").unwrap().unwrap()).unwrap();
    formfield.set_input(&use_header);

    top_app_bar.set_scroll_target(&document.query_selector("main").unwrap().unwrap()).unwrap();
    let drawer2 = drawer.clone();
    let on_nav = Closure::wrap(Box::new(move || {
        drawer2.set_open(!drawer2.open());
    }) as Box<dyn FnMut()>);
    top_app_bar.listen("MDCTopAppBar:nav", &on_nav).unwrap();
    on_nav.forget();

    #[allow(unused_assignments)]
    let mut grid = None;
    let input_file = document.query_selector("input[type=file]").unwrap().unwrap();
    let app_save = document.query_selector(".app-save").unwrap().unwrap();
    let app_save = app_save.dyn_ref::<HtmlButtonElement>().unwrap();

    let mut events = EventStream::new(&[
        (input_file.as_ref(), FileChanged, "change"),
        (app_save.as_ref(), Save, "click"),
    ][..]);

    app_save.set_disabled(true);
    while let Some((token, event)) = events.next().await {
        match token {
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
                    grid = Some(load(&root, &text, use_header.checked()).unwrap());
                    drawer.set_open(false);
                    app_save.set_disabled(false);
                };
            }
            Save => {
                if let Some(grid) = &grid {
                    let csv = grid.csv.lock().await;
                    let content = format!("{}", &*csv);
                    let parts = Array::of1(&JsValue::from(content));
                    let blob = Blob::new_with_str_sequence_and_options(
                        &parts,
                        BlobPropertyBag::new().type_("text/csv")).unwrap();
                    let url = Url::create_object_url_with_blob(&blob).unwrap();
                    window.open_with_url_and_target(&url, "_blank").ok();
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
