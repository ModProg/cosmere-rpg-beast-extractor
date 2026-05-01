use std::fs;
use std::path::PathBuf;

use clap::Parser;
use extract_beasts::pages;
use itertools::Either;
use rayon::iter::{ParallelBridge, ParallelIterator};

mod pdf;
mod structure;
mod text;

#[cfg(test)]
#[cfg(copyrighted)]
mod test_copyrighted;

#[derive(clap::Parser)]
struct Command {
    #[clap(short = 'i', long)]
    pdf: PathBuf,
    #[clap(short, long)]
    out_dir: PathBuf,
    #[clap(short, long)]
    format: Format,
    /// Pages that should be parsed e.g. `12-13,15,22-23` or one of the
    /// predifined ranges:
    /// - `stormlight-worldguide` (191-269)
    /// - `stonewalkers` (136-170)
    #[clap(
        short,
        long,
        help = "Pages that should be parsed e.g. `12-13,15,22` or one of the predefined ranges \
                (see --help)."
    )]
    pages: String,
}

#[derive(clap::ValueEnum, Clone, Copy, PartialEq, Eq)]
enum Format {
    Raw,
    Yaml,
    ObsidianFrontmatter,
}

pub fn main() -> anyhow::Result<()> {
    let Command {
        pdf,
        out_dir,
        pages,
        format,
    } = Command::parse();
    let pages = match pages.as_str() {
        "stormlight-worldguide" => Either::Left(pages::STORMLIGHT_WORLDGUIDE).par_bridge(),
        "stonewalkers" => Either::Left(pages::STONEWALKERS).par_bridge(),
        pages => Either::Right(
            pages
                .split(',')
                .map(|s| s.split_once('-').unwrap_or((s, s)))
                .flat_map(|(f, t)| {
                    let f: u32 = f.trim().parse().unwrap();
                    let t: u32 = t.trim().parse().unwrap();
                    f..=t
                }),
        )
        .par_bridge(),
    };

    let pdf = std::fs::read(pdf).unwrap();

    pdf::extract_pages(pdf, pages).try_for_each(|(page, out)| {
        if format == Format::Raw {
            fs::write(out_dir.join(format!("{page}.txt")), out)?;
        } else {
            let beasts = text::parse(&out);
            for beast in beasts {
                match format {
                    Format::Raw => unreachable!("handled above"),
                    Format::Yaml => fs::write(
                        out_dir.join(format!("{}.yaml", &beast.name)),
                        beast.to_yaml(),
                    )?,
                    Format::ObsidianFrontmatter => fs::write(
                        out_dir.join(format!("{}.md", &beast.name)),
                        beast.into_obsidian_frontmatter(),
                    )?,
                }
            }
        }
        anyhow::Ok(())
    })
}
