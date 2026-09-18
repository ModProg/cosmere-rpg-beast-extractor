use anyhow::bail;
use gloo::utils::format::JsValueSerdeExt;
use gloo::utils::iter::UncheckedIter;
use serde_json::json;

use super::*;
use crate::extractor::{extract_from_page, get_page_text};
use crate::form::get_pages;

def_elem_fn!(el_editor: HtmlElement = "editor");
def_elem_fn!(el_preview_page_select: HtmlSelectElement = "preview-page");
def_elem_fn!(el_editor_style: HtmlSelectElement = "editor-style");
def_elem_fn!(el_data_mapping: HtmlSelectElement = "data-mapping");
def_elem_fn!(el_editor_mapping: HtmlElement = "editor-mapping");
def_elem_fn!(el_parser_grammar: HtmlTextAreaElement = "parser-grammar");
def_elem_fn!(el_parser_output: HtmlElement = "parser-output");

#[wasm_bindgen]
extern "C" {
    type EditorView;

    #[wasm_bindgen(js_namespace = CodeMirror, js_name = fromTextArea)]
    fn from_text_area(elem: &HtmlTextAreaElement, options: JsValue) -> EditorView;

    // static githubDark;
}

pub fn setup() {
    // el_parser_grammar().set_value(
    //     LocalStorage::get::<String>("grammar")
    //         .as_deref()
    //         .unwrap_or(""),
    // );

    // el_parser_grammar().on("input", || {
    //     update_preview_parser();
    // });

    log!(from_text_area(
        &el_parser_grammar(),
        JsValue::from_serde(&json!({
            "lineNumbers": true,
            "theme": "xq-dark"
        }))
        .unwrap()
    ));

    el_data_mapping().on("change", update_mapper_visibility);

    el_editor_style().on("change", update_editor_layout);
}

fn update_editor_layout() {
    let editor_style = el_editor_style().value();
    let class_list = el_editor().class_list();
    if editor_style == "side-by-side" && !class_list.contains("wide") {
        class_list.add_1("wide").unwrap();
    }
    if editor_style == "vertical" && class_list.contains("wide") {
        class_list.remove_1("wide").unwrap();
    }
}

fn update_mapper_visibility() {
    let data_mapping = el_data_mapping().value();
    let style = el_editor_mapping().style();
    if data_mapping == "none" {
        style.set_property("display", "none").unwrap();
    } else {
        style.remove_property("display").unwrap();
    }
}

pub fn show_editor(show: bool) {
    let editor_style = el_editor().style();
    let editor_options = window()
        .expect("window should be present")
        .document()
        .expect("document should be present")
        .query_selector_all(".editor-options")
        .unwrap();

    if show {
        update_editor_layout();
        update_mapper_visibility();
        editor_style.remove_property("display").unwrap();
        for editor_option in UncheckedIter::from(editor_options.values()) {
            editor_option
                .unchecked_into::<HtmlElement>()
                .style()
                .remove_property("display")
                .unwrap();
        }

        update_preview();
    } else {
        editor_style.set_property("display", "none").unwrap();

        for editor_option in UncheckedIter::from(editor_options.values()) {
            editor_option
                .unchecked_into::<HtmlElement>()
                .style()
                .set_property("display", "none")
                .unwrap();
        }
    }
}

pub fn update_preview() {
    if form::el_parser().value() == "manual" {
        update_preview_page_selection();
        update_preview_parser();
    }
}

fn update_preview_page_selection() {
    let el_preview_page_select = el_preview_page_select();
    if let Some(pages) = get_pages() {
        for page in pages.take(100) {
            el_preview_page_select
                .add_with_html_option_element(
                    &HtmlOptionElement::new_with_text(&page.to_string()).unwrap(),
                )
                .unwrap();
        }
    }
}

fn update_preview_parser() {
    spawn_local(async move {
        if let Err(e) = async {
            let Ok(page) = editor::el_preview_page_select().value().parse() else {
                bail!("Select a page to preview")
            };
            let grammar = el_parser_grammar().value();
            el_parser_output().set_inner_text("evaluating...");
            let page = get_page_text(page).await?;
            element_by_id!("parser-input": HtmlTextAreaElement).set_value(&page);
            let text = extract_from_page(&grammar, &page)?;
            el_parser_output().set_inner_text(&text);
            LocalStorage::set("grammar", grammar).unwrap();
            anyhow::Ok(())
        }
        .await
        {
            el_parser_output().set_inner_text(&format!("{e:#}"));
        };
    });
}
