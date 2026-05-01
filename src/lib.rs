mod pdf;

use derive_more::{Display, Error};
pub use pdf::extract_pages;
#[cfg(feature = "rayon")]
pub use pdf::extract_pages_rayon;

pub mod structure;

mod text;
pub use text::parse_page;

pub use crate::structure::Beast;

pub mod pages {
    pub fn resolve(s: &str) -> &str {
        match s {
            "stormlight-worldguide" => STORMLIGHT_WORLDGUIDE,
            "stonewalkers" => STONEWALKERS,
            o => o,
        }
    }
    pub const STORMLIGHT_WORLDGUIDE: &str = "191-269";
    pub const STONEWALKERS: &str = "136-170";
}

impl Beast {
    pub fn to_yaml(&self) -> String {
        yaml_serde::to_string(self).unwrap()
    }

    pub fn into_obsidian_frontmatter(self) -> String {
        format!(
            "---\nstatblock: true\n{}\n---\n",
            self.update_for_obsidian().to_yaml()
        )
    }
}

/// Contains the offending portion of the string that was expected to be a u32
#[derive(Error, Display, Debug)]
#[display("Expected unsigned integer, found: {_0}")]
#[error(ignore)]
pub struct ParsePageError(pub String);
pub fn parse_pages(s: &str) -> Result<impl Iterator<Item = u32>, ParsePageError> {
    Ok(pages::resolve(s)
        .split(',')
        .map(|s| s.split_once('-').unwrap_or((s, s)))
        .map(|(f, t)| {
            let f: u32 = f
                .trim()
                .parse()
                .map_err(|_| ParsePageError(f.trim().to_string()))?;
            let t: u32 = t
                .trim()
                .parse()
                .map_err(|_| ParsePageError(t.trim().to_string()))?;
            Ok(if f > t { t..=f } else { f..=t })
        })
        .collect::<Result<Vec<_>, ParsePageError>>()?
        .into_iter()
        .flatten())
}
