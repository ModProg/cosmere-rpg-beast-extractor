use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_pages(s: &str) -> Result<Vec<u32>, String> {
    Ok(extract_beasts::parse_pages(s)
        .map_err(|e| e.to_string())?
        .take(500)
        .collect())
}

#[wasm_bindgen]
pub fn validate_pages(s: &str) -> Option<String> {
    extract_beasts::parse_pages(s)
        .map_err(|e| e.to_string())
        .err()
}
