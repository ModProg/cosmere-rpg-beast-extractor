use std::fs;
use std::path::PathBuf;

use clap::Parser;
use extract_beasts::{parse_page, parse_pages};
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
    let pages = parse_pages(&pages)?.par_bridge();

    let pdf = std::fs::read(pdf).unwrap();

    pdf::extract_pages_rayon(pdf, pages).try_for_each(|(page, out)| {
        if format == Format::Raw {
            fs::write(out_dir.join(format!("{page}.txt")), out)?;
        } else {
            let beasts = parse_page(&out);
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
