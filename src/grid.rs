use std::sync::Arc;

use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::{future_to_promise, spawn_local};
use js_sys::{Object, Array, Promise, Reflect};
use web_sys::Element;
use futures::lock::Mutex;
use csvparser::Csv;

use crate::sys::cheetah_grid;

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

#[derive(Debug)]
pub(crate) struct Grid {
    name: String,

    csv: Arc<Mutex<Csv>>,

    #[allow(dead_code)]
    grid: cheetah_grid::ListGrid,

    #[allow(dead_code)]
    get_record: Closure<dyn FnMut(usize) -> Promise>,

    #[allow(dead_code)]
    on_changed: Closure<dyn FnMut(Object)>,
}

impl Grid {
    pub(crate) fn new(element: Element, name: String, data: &str, use_header: bool) -> Result<Grid, JsValue> {
        load(element, name, data, use_header)
    }

    pub(crate) fn csv(&self) -> Arc<Mutex<Csv>> {
        self.csv.clone()
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn grid(&self) -> &cheetah_grid::ListGrid {
        &self.grid
    }
}

fn header(csv: &Csv, editor: &cheetah_grid::InlineInputEditor) -> Array {
    vec![js! {
        "field" => "n",
        "caption" => "#",
        "sort" => true,
        "width" => "auto",
        "columnType" => "number"
    }].into_iter().chain((0..csv.max_cols()).map(|i| js! {
        "field" => format!("c{}", i),
        "caption" => csv.header(i).unwrap_or(&format!("{}", i)),
        "action" => editor.clone(),
        "width" => "auto",
        "sort" => true,
        "columnType" => if csv.cols(i).all(|v| v.parse::<f64>().is_ok()) { "number" } else { "text" }
    })).collect()
}

fn load(element: Element, name: String, data: &str, use_header: bool) -> Result<Grid, JsValue> {
    let editor = cheetah_grid::InlineInputEditor::new()?;
    let csv = csvparser::Csv::parse(data, use_header)
        .map_err(|e| format!("failed to parse csv {}", e))?;
    let header = header(&csv, &editor);

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
    })?;

    let opt = js! {
        "parentElement" => element,
        "header" => &header,
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
    let grid = cheetah_grid::ListGrid::new(Some(&opt))?;

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
                    let csv = csv.clone();
                    spawn_local(async move {
                        let mut csv = csv.lock().await;
                        csv.set_val(row - 1, col - 1, val);
                    })
                }
            }
        }) as Box<dyn FnMut(Object)>)
    };
    grid.listen(&cheetah_grid::CHANGED_VALUE, &on_changed).unwrap();

    Ok(Grid { get_record, name, csv, grid, on_changed, })
}
