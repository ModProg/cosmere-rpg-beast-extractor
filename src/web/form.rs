use anyhow::bail;

use super::*;

def_elem_fn!(el_select_pages: HtmlSelectElement = "predefined-page-ranges");
def_elem_fn!(el_parser: HtmlSelectElement = "parser");
def_elem_fn!(el_pages: HtmlInputElement = "pages");
def_elem_fn!(el_form: HtmlElement = "form");

pub fn get_pages() -> Option<impl Iterator<Item = u32>> {
    let el_pages = el_pages();
    let pages = match parse_pages(&el_pages.value()) {
        Err(error) => {
            log!(error.to_string());
            el_pages.set_custom_validity(&format!("Pages should match `1,2-4,...`: {error}"));
            None
        }
        Ok(pages) => {
            el_pages.set_custom_validity("");
            Some(pages)
        }
    };
    el_pages.report_validity();
    pages
}

pub fn setup() {
    el_select_pages().on("change", || {
        let selected_pages = el_select_pages().value();
        element_by_id!("pages": HtmlInputElement).set_value(pages::resolve(&selected_pages));
        editor::update_preview();
    });

    el_pages().on("input", || {
        get_pages();
    });
    el_pages().on("blur", || {
        editor::update_preview();
    });

    el_parser().on("change", || {
        editor::show_editor(el_parser().value() == "manual");
    });
    editor::show_editor(el_parser().value() == "manual");

    el_form().show(true);
}

pub fn get_file() -> anyhow::Result<gloo::file::File> {
    let file = element_by_id!("file": HtmlInputElement);
    if let Some(file) = file.files().unwrap().get(0) {
        // Ok(read_as_bytes(file).await.unwrap())
        Ok(file.into())
    } else {
        bail!("No File selected.");
    }
}
