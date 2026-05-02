//! To enable these tests add `--cfg=copyrighted` to `RUSTCFLAGS`. It is also
//! recomended to run in `--release` as this speeds up tests enormously.
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
use rayon::iter::ParallelIterator;

use crate::*;

#[test]
fn stormlight_world_guide() {
    test("stormlight-worldguide");
}

#[test]
fn stonewalkers() {
    test("stonewalkers");
}

fn test(name: &'static str) {
    insta::with_settings!({
        prepend_module_to_snapshot=>false,
        snapshot_path=>format!("../copyrighted/{name}/snapshots")
    }, {
        let index: BTreeMap<u32, Vec<String>> = pdf::extract_pages_rayon(
            fs::read(format!("copyrighted/{name}/{name}.pdf")).unwrap(),
            parse_pages(name).unwrap().par_bridge(),
        )
        .map(|(page, content)| {
            insta::with_settings!({
                prepend_module_to_snapshot=>false,
                snapshot_path=>format!("../copyrighted/{name}/snapshots")
            }, {

                assert_snapshot!(format!("text-{page}"), &content);
                let parsed = parse_page(&content);
                let names = parsed.iter().map(|b|b.name.clone()).collect::<Vec<_>>();
                assert_yaml_snapshot!(format!("parsed-{page}.yaml"), parsed);
                for beast in parsed {
                    assert_snapshot!(
                        format!("obsidian-frontmatter-{page}-{}.md", beast.name.replace(' ', "_")),
                        beast.into_obsidian_frontmatter()
                    );
                }
                (page, names)
            })
        })
        .collect();

        assert_yaml_snapshot!("index.yaml", index);
    })
}
