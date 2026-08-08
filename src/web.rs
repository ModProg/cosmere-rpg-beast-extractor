use std::io::Write;
use std::{io, iter};

use extract_beasts::{Parser, extract_page, extract_pages, pages, parse_page_old, parse_pages};
use gloo::console::{Timer, log};
use gloo::file::futures::read_as_bytes;
use gloo::file::{Blob, FileList};
use gloo::storage::{LocalStorage, Storage};
use gloo::utils::document;
use itertools::Either;
use wasm_bindgen::convert::{FromWasmAbi, IntoWasmAbi};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::*;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

macro_rules! element_by_id {
    ($name:literal $(: $type:ty)?) => {{
        let document = window().expect("window should be present").document().expect("document should be present");
        document
            .get_element_by_id($name)
            .expect(concat!("element #", $name, " should exist"))
            $(.dyn_into::<$type>().expect(concat!("element #", $name, " should be a ", stringify!($type))))?
    }};
}

fn callback<T: FromWasmAbi, O: IntoWasmAbi, F: 'static + FnMut(T) -> O>(
    fun: F,
) -> ScopedClosure<'static, dyn FnMut(T) -> O> {
    ScopedClosure::new(fun)
}

#[wasm_bindgen(main)]
async fn main() {
    console_error_panic_hook::set_once();

    let select_pages_callback = callback(|_: Event| {
        let selected_pages = element_by_id!("predefined-page-ranges": HtmlSelectElement).value();
        element_by_id!("pages": HtmlInputElement).set_value(pages::resolve(&selected_pages));
    });
    element_by_id!("predefined-page-ranges")
        .add_event_listener_with_callback("change", select_pages_callback.as_ref().unchecked_ref())
        .unwrap();
    select_pages_callback.forget();

    let pages_input_callback = callback(|_: Event| {
        let pages = element_by_id!("pages": HtmlInputElement);
        if let Err(error) = parse_pages(&pages.value()) {
            pages.set_custom_validity(&format!("Pages should match `1,2-4,...`: {error}"));
        } else {
            pages.set_custom_validity("");
        }
    });
    element_by_id!("pages": HtmlInputElement)
        .add_event_listener_with_callback("input", pages_input_callback.as_ref().unchecked_ref())
        .unwrap();
    pages_input_callback.forget();

    let grammar_input = element_by_id!("parser-grammar": HtmlTextAreaElement);
    grammar_input.set_value(
        LocalStorage::get::<String>("grammar")
            .as_deref()
            .unwrap_or(""),
    );
    let parser_input_callback = callback(|_: Event| {
        spawn_local(async {
            convert_one_page().await;
            let grammar = &element_by_id!("parser-grammar": HtmlTextAreaElement).value();
            LocalStorage::set("grammar", grammar).unwrap();
        })
    });
    grammar_input
        .add_event_listener_with_callback("input", parser_input_callback.as_ref().unchecked_ref())
        .unwrap();
    parser_input_callback.forget();

    let parse_pdf_callback = callback(|e: Event| {
        e.prevent_default();
        spawn_local(async {
            let form = element_by_id!("form");
            form.set_attribute("data-disabled", "").unwrap();
            let loading = element_by_id!("loading");
            loading.remove_attribute("data-disabled").unwrap();
            convert_pdf().await;
            form.remove_attribute("data-disabled").unwrap();
            loading.set_attribute("data-disabled", "").unwrap();
        });
    });

    element_by_id!("form")
        .add_event_listener_with_callback("submit", parse_pdf_callback.as_ref().unchecked_ref())
        .unwrap();
    parse_pdf_callback.forget();
}

async fn convert_one_page() {
    element_by_id!("parser-output": HtmlElement).set_inner_text("huh");
    let grammar = &element_by_id!("parser-grammar": HtmlTextAreaElement).value();
    let parser = match Parser::new(grammar) {
        Ok(p) => p,
        Err(e) => {
            element_by_id!("parser-output": HtmlElement)
                .set_inner_text(&format!("Error found while {e:?}"));
            return;
        }
    };
    log!("hi");

    let pages = &element_by_id!("pages": HtmlInputElement).value();
    let pages = parse_pages(pages)
        .expect("validated on input")
        .next()
        .unwrap();
    // let format = element_by_id!("format": HtmlSelectElement).value();
    let file = element_by_id!("file": HtmlInputElement);
    if let Some(file) = FileList::from(file.files().unwrap()).first() {
        let data = read_as_bytes(file).await.unwrap();
        // let from = std::time::Instant::now();
        let timer = Timer::new("extract_pages");
        let page = extract_page(data, pages);
        drop(timer);
        // log!("took", format!("{:?}", from.elapsed()));
        element_by_id!("parser-input": HtmlTextAreaElement).set_value(&page);
        let result = parser.parse_page(&page);
        // let result = parse_page(&page, grammar);
        let result = match result {
            Ok(o) => yaml_serde::to_string(&o).unwrap(),
            Err(e) => e.to_string(),
        };
        element_by_id!("parser-output": HtmlElement).set_inner_text(&result);
    } else {
        element_by_id!("parser-output": HtmlElement).set_inner_text("No File selected.");
    }
}
async fn convert_pdf() {
    let pages = &element_by_id!("pages": HtmlInputElement).value();
    let pages = parse_pages(pages).expect("validated on input");
    let format = element_by_id!("format": HtmlSelectElement).value();
    let file = element_by_id!("file": HtmlInputElement);
    if let Some(file) = FileList::from(file.files().unwrap()).first() {
        let data = read_as_bytes(file).await.unwrap();
        let pages = extract_pages(data, pages).flat_map(|(page, content)| {
            if format == "raw" {
                Either::Left(iter::once((format!("{page}.txt"), content)))
            } else {
                Either::Right(parse_page_old(&content).into_iter().map(
                    |beast| match format.as_ref() {
                        "yaml" => (format!("{}.yaml", beast.name), beast.to_yaml()),
                        "obsidian-frontmatter" => (
                            format!("{}.md", beast.name),
                            beast.into_obsidian_frontmatter(),
                        ),
                        _ => unreachable!("{format}"),
                    },
                ))
            }
        });
        let mut out = io::Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut out);
        for (name, content) in pages {
            zip.start_file(name, SimpleFileOptions::default()).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        let blob = Blob::new_with_options(&*out.into_inner(), Some("application/zip"));
        let url = Url::create_object_url_with_blob(&blob.into()).unwrap();

        let a = document()
            .create_element("a")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        a.set_attribute("href", &url).unwrap();
        let file_name = file.name();
        let file_name = file_name.strip_suffix(".pdf").unwrap_or(&file_name);
        a.set_attribute("download", &format!("{file_name}-beasts.zip"))
            .unwrap();
        a.click();
        Url::revoke_object_url(&url).unwrap();
    } else {
        todo!("no file found");
    }
}
