use std::io::Write;
use std::{io, iter};

use extract_beasts::{extract_pages, pages, parse_page, parse_pages};
use gloo::file::futures::read_as_bytes;
use gloo::file::{Blob, FileList};
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

async fn convert_pdf() {
    let pages = &element_by_id!("pages": HtmlInputElement).value();
    let pages = parse_pages(pages).expect("validated on input");
    let format = element_by_id!("format": HtmlSelectElement).value();
    let file = element_by_id!("file": HtmlInputElement);
    if let Some(file) = FileList::from(file.files().unwrap()).first() {
        let data = read_as_bytes(file).await.unwrap();
        let pages =
            extract_pages(data, pages).flat_map(|(page, content)| {
                if format == "raw" {
                    Either::Left(iter::once((format!("{page}.txt"), content)))
                } else {
                    Either::Right(parse_page(&content).into_iter().map(
                        |beast| match format.as_ref() {
                            "yaml" => (format!("{}.yaml", beast.name), beast.to_yaml()),
                            "obsidian-frontmatter" => (
                                format!("{}.md", beast.name),
                                beast.into_obsidian_frontmatter(),
                            ),
                            _ => unreachable!(),
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
        todo!();
    }
}
