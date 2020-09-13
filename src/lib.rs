use std::pin::Pin;
use std::task::{Poll, Context};
use std::collections::BTreeMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, EventTarget, Element, HtmlInputElement};
use js_sys::{Object, Array};
use gloo::events::EventListener;
use futures::stream::{Stream, StreamExt};
use futures::channel::mpsc::{self, Receiver};

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

struct EventStream {
    queue: Receiver<Event>,
    _listeners: Vec<EventListener>,
}

impl EventStream {
    fn new(target: &EventTarget, events: &[&'static str]) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let mut listeners = vec![];
        for event in events {
            let mut tx = tx.clone();
            let listener = EventListener::new(target, *event, move |event| {
                tx.start_send(event.clone()).unwrap();
            });
            listeners.push(listener);
        }
        Self {
            queue: rx,
            _listeners: listeners,
        }
    }
}

impl Stream for EventStream {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self { ref mut queue, .. } = self.get_mut();
        let event = Pin::new(queue).poll_next(cx);
        match event {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(event)) => Poll::Ready(Some(event)),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}

fn load(root: &Element, data: &[u8], use_header: bool) -> Result<(), JsValue> {
    let document = root.owner_document().unwrap();
    let div = &document.create_element("div").unwrap();
    if let Some(old) = root.first_element_child() {
        old.replace_with_with_node_1(&div).unwrap();
    } else {
        root.append_child(&div).unwrap();
    };

    let inline_input = cheetah_grid::InlineInputEditor::new().unwrap();
    let mut header = BTreeMap::new();

    let mut reader = csv::ReaderBuilder::new().has_headers(use_header).from_reader(data);
    if use_header {
        for (i, cell) in reader.headers().unwrap().iter().enumerate() {
            header.insert(i, (js! {
                "field" => format!("c{}", i),
                "caption" => cell,
                "action" => inline_input.clone(),
                "width" => "auto",
                "sort" => true
            }, "number"));
        }
    }
    let records = Array::new();
    for (n, result) in reader.records().enumerate() {
        let result = result.unwrap();
        let row = js! {
            "n" => format!("{}", n)
        };
        for (i, cell) in result.iter().enumerate() {
            if !header.contains_key(&i) {
                header.insert(i, (js! {
                    "field" => format!("c{}", i),
                    "caption" => format!("{}", i),
                    "action" => inline_input.clone(),
                    "width" => "auto",
                    "sort" => true
                }, "number"));
            }

            if cell.parse::<f64>().is_err() {
                header.get_mut(&i).unwrap().1 = "string";
            }

            Object::assign(&row, &js! {
                format!("c{}", i) => cell
            });
        }
        records.push(&row);
    }

    for (h, t) in header.values() {
        Object::assign(&h, &js! {
            "columnType" => *t
        });
    };

    let header: Vec<Object> = vec![js! {
        "field" => "n",
        "caption" => "#",
        "sort" => true,
        "width" => "auto",
        "columnType" => "number"
    }].into_iter().chain(header.values().map(|(h, _)| h.clone())).collect::<Vec<_>>();

    let opt = js! {
        "parentElement" => div,
        "header" => &header.iter().collect::<Array>(),
        "records" => &records,
        "font" => "16px monospace",
        "allowRangePaste" => true,
        "keyboardOptions" => &js! {
            "moveCellOnTab" => true,
            "moveCellOnEnter" => true,
            "deleteCellValueOnDel" => true,
            "selectAllOnCtrlA" => true
        }
    };
    cheetah_grid::ListGrid::new(Some(&opt)).unwrap();

    Ok(())
}

async fn async_main() -> Result<(), JsValue> {
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

    let input_file = document.query_selector("input[type=file]").unwrap().unwrap();
    let mut events = EventStream::new(&input_file, &["change"]);
    while let Some(event) = events.next().await {
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
            let bytes = if let Some(coder) = coder {
                coder.decode(&bytes, encoding::DecoderTrap::Ignore).map(String::into_bytes).unwrap_or(bytes)
            } else {
                bytes
            };
            load(&root, &bytes, use_header.checked()).unwrap();
            drawer.set_open(false);
        };
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
