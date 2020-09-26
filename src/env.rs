use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{
    Location,
    Element,
    HtmlInputElement,
    HtmlButtonElement,
};
use crate::sys::material::{
    MDCTopAppBar,
    MDCDrawer,
    MDCFormField,
    MDCCheckbox,
    MDCMenu,
};

#[derive(Debug)]
pub(crate) struct Env {
    location: Location,
    root: Element,
    input_file: HtmlInputElement,
    menu: Element,
    header: Element,
    app_save: HtmlButtonElement,
    app_use_header: HtmlInputElement,
    error: Element,

    mdc_drawer: MDCDrawer,
    mdc_menu: MDCMenu,
}

impl Env {
    pub(crate) fn initialize() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window found.")?;
        let location = window.location();
        let document = window.document().ok_or("No document found.")?;
        let root = document.query_selector("main")?.ok_or("No root found.")?;
        let header = document.query_selector("header")?.ok_or("No header found")?;
        let aside = document.query_selector("aside")?.ok_or("No aside found")?;

        let input_file = document.query_selector("input[type='file']")?.ok_or("Element not found")?
            .dyn_into::<HtmlInputElement>()?;
        let app_save = document.query_selector(".app-save")?.ok_or("Element not found")?
            .dyn_into::<HtmlButtonElement>()?;
        let app_use_header = document.query_selector(".app-use-header")?.ok_or("Element not found")?
            .dyn_into::<HtmlInputElement>()?;
        let form_field_use_header = document.query_selector(".mdc-form-field")?.ok_or("Element not found")?;
        let menu = document.query_selector(".mdc-menu")?.ok_or("Element not found")?;
        let error = document.query_selector("#error")?.ok_or("Element not found")?;

        MDCTopAppBar::new(&header)?.set_scroll_target(&header)?;
        let mdc_drawer = MDCDrawer::attachTo(&aside)?;
        MDCFormField::new(&form_field_use_header)?.set_input(
            MDCCheckbox::new(&app_use_header.parent_element().ok_or("Element not found")?)?.as_ref());
        let mdc_menu = MDCMenu::new(&menu)?;

        Ok(Self {
            location,
            root,
            input_file,
            menu,
            error,
            header,
            app_save,
            app_use_header,
            mdc_drawer,
            mdc_menu,
        })
    }

    pub(crate) fn location(&self) -> &Location {
        &self.location
    }

    pub(crate) fn root(&self) -> &Element {
        &self.root
    }

    pub(crate) fn header(&self) -> &Element {
        &self.header
    }

    pub(crate) fn input_file(&self) -> &HtmlInputElement {
        &self.input_file
    }

    pub(crate) fn app_save(&self) -> &HtmlButtonElement {
        &self.app_save
    }

    pub(crate) fn app_use_header(&self) -> &HtmlInputElement {
        &self.app_use_header
    }

    pub(crate) fn error(&self) -> &Element {
        &self.error
    }

    pub(crate) fn menu(&self) -> &Element {
        &self.menu
    }

    pub(crate) fn mdc_drawer(&self) -> &MDCDrawer {
        &self.mdc_drawer
    }

    pub(crate) fn mdc_menu(&self) -> &MDCMenu {
        &self.mdc_menu
    }

}
