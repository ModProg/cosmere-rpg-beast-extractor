use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::Context;
use gloo::file::File;
use gloo::storage::SessionStorage;

use super::*;
use crate::form::get_file;

async fn cache_key(file: &File, page: u32) -> String {
    let mut hasher = DefaultHasher::new();
    // TODO think about a better sampling :)
    let size = file.size();
    read_as_bytes(&file.slice(0, size.min(1000)))
        .await
        .unwrap()
        .hash(&mut hasher);
    read_as_bytes(&file.slice(size.max(1000) - 1000, size))
        .await
        .unwrap()
        .hash(&mut hasher);
    format!("{}:{}", hasher.finish(), page)
}

fn get_cache(key: &str) -> Option<String> {
    SessionStorage::get(key).ok()
}

fn set_cache(key: &str, page: &str) {
    SessionStorage::set(key, page).unwrap();
}

pub async fn get_page_text(page: u32) -> anyhow::Result<String> {
    let file = get_file()?;
    let cache_key = cache_key(&file, page).await;
    Ok(if let Some(cached) = get_cache(&cache_key) {
        cached
    } else {
        let data = read_as_bytes(&file).await.unwrap();
        let page = extract_page(data, page);
        set_cache(&cache_key, &page);
        page
    })
}

pub fn extract_from_page(grammar: &str, page: &str) -> anyhow::Result<String> {
    let parser = Parser::new(grammar).context("Error found while parsing grammar:")?;

    parser
        .parse_page(page)
        .and_then(|v| yaml_serde::to_string(&v).map_err(anyhow::Error::from))
}
