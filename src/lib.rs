use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlElement,
    HtmlInputElement,
    Event,
    MouseEvent,
    DragEvent,
    CustomEvent,
    Url,
    File,
    FilePropertyBag,
    DataTransfer,
};
use js_sys::{Array, Reflect, Error as JsError, Uint8Array};
use futures::stream::StreamExt as _;
use encoding::EncodingRef;

use event_stream::EventStream;
use env::Env;

mod sys;
mod event_stream;
mod env;
mod grid;

struct State {
    env: Env,
    grid: Option<grid::Grid>,
    coder: Option<EncodingRef>,
}

async fn load_csv(file: &gloo::file::File, state: &mut State) -> Result<(), JsValue> {
    use gloo::file::futures::read_as_bytes;
    use encoding::label::encoding_from_whatwg_label;

    let bytes = read_as_bytes(file).await.map_err(|e| format!("failed to read file {}", e))?;

    let (encoding, _, _) = chardet::detect(&bytes);
    let mut using_coder = encoding_from_whatwg_label(chardet::charset2encoding(&encoding));
    let text = if let Some(coder) = using_coder {
        coder.decode(&bytes, encoding::DecoderTrap::Replace).ok()
    } else {
        None
    };
    let text = if let Some(text) = text {
        text
    } else {
        using_coder = None;
        String::from_utf8_lossy(&bytes).to_string()
    };

    let State { env, ref mut grid, ref mut coder } = state;

    let root = env.root();
    let document = root.owner_document().ok_or("no owner document found")?;
    let div = document.create_element("div")?;
    if let Some(old) = root.first_element_child() {
        old.replace_with_with_node_1(&div)?;
    } else {
        root.append_child(&div)?;
    };

    *grid = Some(grid::Grid::new(div, file.name(), &text, env.app_use_header().checked())?);
    *coder = using_coder;

    env.mdc_drawer().set_open(false);
    env.app_save().set_disabled(false);
    Ok(())
}

#[derive(Debug, Clone)]
enum EventType {
    FileChanged,
    Save,
    ContextMenu,
    MenuSelected,
    AppBarNav,
    DragOver,
    Drop,
}

impl EventType {
    async fn handle(&self, event: &Event, state: &mut State) -> Result<(), JsValue> {
        match self {
            Self::FileChanged => self.handle_file_changed(event, state).await,
            Self::Save => self.handle_save(event, state).await,
            Self::ContextMenu => self.handle_context_menu(event, state).await,
            Self::MenuSelected => self.handle_menu_selected(event, state).await,
            Self::AppBarNav => self.handle_app_bar_nav(event, state).await,
            Self::DragOver => self.handle_drag_over(event, state).await,
            Self::Drop => self.handle_drop(event, state).await,
        }
    }

    async fn handle_drag_over(&self, event: &Event, _state: &mut State) -> Result<(), JsValue> {
        event.prevent_default();
        Ok(())
    }

    async fn handle_drop(&self, event: &Event, state: &mut State) -> Result<(), JsValue> {
        event.prevent_default();
        let file_list = event.dyn_ref::<DragEvent>()
            .and_then(DragEvent::data_transfer)
            .as_ref()
            .and_then(DataTransfer::files)
            .map(gloo::file::FileList::from);
        let file_list = if let Some(file_list) = file_list {
            file_list
        } else {
            return Ok(())
        };

        if let Some(file) = file_list.iter().next() {
            load_csv(file, state).await?;
        };
        Ok(())
    }

    async fn handle_file_changed(&self, event: &Event, state: &mut State) -> Result<(), JsValue> {
        let file_list = event.target()
            .as_ref().and_then(JsCast::dyn_ref::<HtmlInputElement>)
            .and_then(HtmlInputElement::files)
            .map(gloo::file::FileList::from);
        let file_list = if let Some(file_list) = file_list {
            file_list
        } else {
            return Ok(())
        };

        if let Some(file) = file_list.iter().next() {
            load_csv(file, state).await?;
        };
        Ok(())
    }

    async fn handle_save(&self, _event: &Event, state: &mut State) -> Result<(), JsValue> {
        let State { grid, env, coder } = state;
        if let Some(grid) = &grid {
            let csv = grid.csv();
            let csv = csv.lock().await;
            let csv_content = format!("{}", &*csv);
            let content = if let Some(coder) = coder {
                coder.encode(&csv_content, encoding::EncoderTrap::Replace).ok()
            } else {
                None
            };
            let content = content.unwrap_or_else(|| csv_content.as_bytes().to_vec());
            let parts = Array::of1(Uint8Array::from(content.as_ref()).buffer().as_ref());
            let blob = File::new_with_buffer_source_sequence_and_options(
                &parts,
                &grid.name(),
                FilePropertyBag::new().type_("text/csv"))?;
            let url = Url::create_object_url_with_blob(&blob)?;
            env.location().assign(&url)?;
            Url::revoke_object_url(&url)?;
        }
        Ok(())
    }

    async fn handle_context_menu(&self, event: &Event, state: &mut State) -> Result<(), JsValue> {
        let State { grid, env, .. } = state;
        if grid.is_some() {
            let event = event.dyn_ref::<MouseEvent>().ok_or("event type mismatch")?;
            event.prevent_default();
            env.mdc_menu().set_absolute_position(event.client_x(), event.client_y())?;
            env.mdc_menu().set_open(true);
        }
        Ok(())
    }

    async fn handle_menu_selected(&self, event: &Event, state: &mut State) -> Result<(), JsValue> {
        let State { grid, .. } = state;
        let event = event.dyn_ref::<CustomEvent>().ok_or("event type mismatch")?;
        let detail = event.detail();
        #[allow(unused_unsafe)]
        let item = unsafe {
            Reflect::get(&detail, &"item".into())?
        };
        let item = item.dyn_into::<HtmlElement>()?;
        if let (Some(grid), Some(action)) = (&grid, &item.dataset().get("action")) {
            let csv = grid.csv();
            let mut csv = csv.lock().await;
            let grid = grid.grid();

            let selection = grid.selection()?;
            #[allow(unused_unsafe)]
            let row = unsafe {
                let select = Reflect::get(&selection, &"select".into())?;
                Reflect::get(&select, &"row".into())?
            };
            match (action.as_ref(), row.as_f64().map(|f| f as usize)) {
                ("add_before", Some(row)) if (1..=csv.rows()).contains(&row) => csv.insert_row(row - 1),
                ("add_after", Some(row)) if (0..=csv.rows()).contains(&row) => csv.insert_row(row - 0),
                ("remove", Some(row)) if (1..=csv.rows()).contains(&row) => csv.remove_row(row - 1),
                _ => return Ok(()),
            }
            let data_source = grid.data_source()?;
            data_source.set_length(csv.rows())?;
            data_source.clear_cache()?;
            grid.invalidate()?;
        }
        Ok(())
    }

    async fn handle_app_bar_nav(&self, _event: &Event, state: &mut State) -> Result<(), JsValue> {
        let drawer = state.env.mdc_drawer();
        drawer.set_open(!drawer.open());
        Ok(())
    }
}

async fn async_main() -> Result<(), JsValue> {
    use EventType::*;

    let env = Env::initialize()?;
    env.app_save().set_disabled(true);

    let mut events = EventStream::new(&[
        (env.input_file().as_ref(), FileChanged, "change"),
        (env.app_save().as_ref(), Save, "click"),
        (env.root().as_ref(), ContextMenu, "contextmenu"),
        (env.menu().as_ref(), MenuSelected, "MDCMenu:selected"),
        (env.header().as_ref(), AppBarNav, "MDCTopAppBar:nav"),
        (env.root().as_ref(), DragOver, "dragover"),
        (env.root().as_ref(), Drop, "drop"),
    ][..]);
    let mut state = State { env, grid: None, coder: None };

    while let Some((token, event)) = events.next().await {
        if let Err(err) = token.handle(&event, &mut state).await {
            log::error!("{:?}", err);
            if let Some(err) = err.dyn_ref::<JsError>() {
                if let Some(err) = JsValue::from(err.message()).as_string() {
                    state.env.error().set_text_content(Some(&err));
                }
            } else if let Some(err) = err.as_string() {
                state.env.error().set_text_content(Some(&err));
            }
        } else {
            state.env.error().set_text_content(None);
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
