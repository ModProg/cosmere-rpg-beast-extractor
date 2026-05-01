//! To enable these tests add `--cfg=copyrighted` to `RUSTCFLAGS`.
//!
//! 1. run cargo test with `RUSTFLAGS=--cfg=copyrighted cargo insta test`
//! 2. add to `.cargo/config`
//!   ```toml
//!   [build]
//!   rustflags = ["--cfg=copyrighted"]
//!   ```
//!
//! This requires you to have the PDF-Files of the Stormlight Worldguide and the
//! Stonewalkers campaing at:
//! - `copyrighted/stormlight-worldguide/stormlight-worldguide.pdf`
//! - `copyrighted/stonewalkers/stonewalkers.pdf`
use std::collections::BTreeMap;

use insta::{assert_snapshot, assert_yaml_snapshot};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::*;

#[test]
fn stormlight_world_guide() {
    test("stormlight-worldguide", pages::STORMLIGHT_WORLDGUIDE);
}

#[test]
fn stonewalkers() {
    test("stonewalkers", pages::STONEWALKERS);
}

fn test(name: &'static str, pages: impl IntoParallelIterator<Item = u32>) {
    insta::with_settings!({
        prepend_module_to_snapshot=>false,
        snapshot_path=>format!("../copyrighted/{name}/snapshots")
    }, {
        let index: BTreeMap<u32, Vec<String>> = pdf::extract_pages(
            format!("copyrighted/{name}/{name}.pdf"),
            pages.into_par_iter(),
        )
        .map(|(page, content)| {
            insta::with_settings!({
                prepend_module_to_snapshot=>false,
                snapshot_path=>format!("../copyrighted/{name}/snapshots")
            }, {

                assert_snapshot!(format!("text-{page}"), &content);
                let parsed = text::parse(&content);
                assert_yaml_snapshot!(format!("parsed-{page}.yaml"), parsed);
                for beast in &parsed {
                    assert_snapshot!(
                        format!("obsidian-frontmatter-{page}-{}.md", beast.name.replace(' ', "_")),
                        format!("---\n{}\n---", yaml_serde::to_string(beast).unwrap())
                    );
                }
                (page, parsed.iter().map(|b|b.name.clone()).collect::<Vec<_>>())
            })
        })
        .collect();

        assert_yaml_snapshot!("index.yaml", index);
    })
}
