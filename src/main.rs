use std::fs;
use std::path::PathBuf;

use clap::Parser;
use itertools::Either;

mod pdf;
mod structure;
mod text;

#[derive(clap::Parser)]
struct Command {
    #[clap(short = 'i', long)]
    pdf: PathBuf,
    #[clap(short, long)]
    out_dir: PathBuf,
    #[clap(short, long)]
    pages: String,
}

pub fn main() {
    let Command {
        pdf,
        out_dir,
        pages,
    } = Command::parse();
    let pages = pages
        .split(',')
        .map(|s| s.split_once('-').unwrap_or((s, s)))
        .flat_map(|(f, t)| {
            let f: u32 = f.trim().parse().unwrap();
            let t: u32 = t.trim().parse().unwrap();
            f..=t
        });
    let pages = if true {
        Either::Left(
            pdf::extract_pages(pdf, pages)
                .inspect(|(i, s)| fs::write(out_dir.join(format!("{i}.txt")), s).unwrap()),
        )
    } else {
        Either::Right(pages.map(|i| {
            (
                i,
                fs::read_to_string(out_dir.join(format!("{i}.txt"))).unwrap(),
            )
        }))
    };
    for (_, out) in pages {
        let beast = text::parse(&out);
        eprintln!("{beast:#?}");
    }
}

#[test]
fn test() {
    use insta::{assert_snapshot, assert_yaml_snapshot};
    for (page, content) in pdf::extract_pages(
        "copyrighted/stormlight-world-guide/stormlight-world-guide.pdf",
        191..=240,
    ) {
        eprintln!("{page}");
        insta::with_settings!({
            prepend_module_to_snapshot=>false,
            snapshot_path=>"../copyrighted/stormlight-world-guide/snapshots"
        }, {
            assert_snapshot!(format!("text-{page}"), &content);
            assert_yaml_snapshot!(format!("parsed-{page}.yaml"), text::parse(&content));
        })
    }
}
