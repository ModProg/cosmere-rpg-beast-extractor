use std::collections::{HashMap, HashSet};
use std::mem;
use std::path::Path;
use std::sync::Arc;

use euclid::{Transform2D, vec2};
use hayro_syntax::Pdf;
use pdf_extract::{MediaBox, OutputDev, OutputError, Transform};

struct ExtendedPlainTextOutput<'a> {
    writer: &'a mut String,
    last_end: f64,
    last_y: f64,
    first_char: bool,
    flip_ctm: Transform,
    icon_map: &'a HashMap<String, String>,
    pending_icons: String,
}

impl OutputDev for ExtendedPlainTextOutput<'_> {
    fn begin_page(
        &mut self,
        _page_num: u32,
        media_box: &MediaBox,
        _: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), OutputError> {
        self.flip_ctm = Transform2D::row_major(1., 0., 0., -1., 0., media_box.ury - media_box.lly);
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), OutputError> {
        Ok(())
    }

    fn output_character(
        &mut self,
        trm: &Transform,
        width: f64,
        _spacing: f64,
        font_size: f64,
        char: &str,
    ) -> Result<(), OutputError> {
        let position = trm.post_transform(&self.flip_ctm);
        let transformed_font_size_vec = trm.transform_vector(vec2(font_size, font_size));
        let transformed_font_size =
            (transformed_font_size_vec.x * transformed_font_size_vec.y).sqrt();
        let (x, y) = (position.m31, position.m32);
        use std::fmt::Write;
        if self.first_char {
            let y_spacing = (y - self.last_y).abs();
            if self.writer.lines().last().is_some_and(|l| {
                l.ends_with("Graze: 3 (1d6) keen damage. Hit: 8 (1d6 + 5) keen damage.")
            }) {
                eprintln!("{char:?}");
                eprintln!("{:?}", self.writer.lines().last());
                eprintln!("{y} - {} = {y_spacing} > 15.", self.last_y,);
                eprintln!(
                    "{y} - {} = {y_spacing} > {transformed_font_size} * 1.5",
                    self.last_y,
                );
                eprintln!("{x} < {}", self.last_end);
            }
            if y_spacing > 15. {
                write!(self.writer, "\n\n")?;
            } else if y_spacing > 10. {
                writeln!(self.writer)?;
            } else {
                if y_spacing > transformed_font_size * 1.5 {
                    writeln!(self.writer)?;
                }
                if x < self.last_end && y_spacing > transformed_font_size * 0.5 {
                    writeln!(self.writer)?;
                }
            }
            if !self.writer.ends_with(|c: char| c.is_whitespace())
                && x > self.last_end + transformed_font_size * 0.1
            {
                write!(self.writer, " ")?;
            }
        }
        if !self.pending_icons.is_empty() {
            write!(self.writer, "{}", mem::take(&mut self.pending_icons))?;
        }
        write!(self.writer, "{}", char)?;
        self.first_char = false;
        self.last_y = y;
        self.last_end = x + width * transformed_font_size;
        Ok(())
    }

    fn begin_word(&mut self) -> Result<(), OutputError> {
        self.first_char = true;
        Ok(())
    }

    fn end_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }

    fn end_line(&mut self) -> Result<(), OutputError> {
        Ok(())
    }

    fn stroke(
        &mut self,
        trm: &pdf_extract::Transform,
        colorspace: &pdf_extract::ColorSpace,
        color: &[f64],
        path: &pdf_extract::Path,
    ) -> Result<(), OutputError> {
        self.fill(trm, colorspace, color, path)
    }

    fn fill(
        &mut self,
        _trm: &pdf_extract::Transform,
        _colorspace: &pdf_extract::ColorSpace,
        _color: &[f64],
        path: &pdf_extract::Path,
    ) -> Result<(), OutputError> {
        if let Some(icon) = self.icon_map.get(&format!("{path:?}")) {
            // self.output_character(trm, 1.0, 0., 4.5, icon)
            // icons usually should not come last in any expression (at least a trailing `.`
            // should follow).
            self.pending_icons.push_str(icon);
        } else {
            // eprintln!("{path:?}@{_color:?}");
            // self.pending_icons.push_str(&format!("{path:?}@{_color:?}"))
        }
        self.begin_word()
    }
}

fn extract_text_by_page(
    pdf: &Pdf,
    page_num: u32,
    icon_map: &HashMap<String, String>,
) -> Result<String, OutputError> {
    let mut s = String::new();
    {
        let mut output = ExtendedPlainTextOutput {
            writer: &mut s,
            icon_map,
            last_end: 100000.,
            first_char: false,
            last_y: 0.,
            flip_ctm: Transform2D::identity(),
            pending_icons: String::new(),
        };
        pdf_extract::output_doc_page(pdf, &mut output, page_num)?;
    }
    Ok(s)
}

pub fn extract_pages(
    path: impl AsRef<Path>,
    pages: impl IntoIterator<Item = u32>,
) -> impl Iterator<Item = (u32, String)> {
    let icon_map: HashMap<String, HashSet<String>> =
        serde_yaml_ng::from_str(include_str!("icon_map.yaml")).unwrap();
    // required because of yaml key length limit
    let icon_map = icon_map
        .into_iter()
        .flat_map(|(a, b)| b.into_iter().map(move |b| (b, a.clone())))
        .collect();
    let bytes = std::fs::read(path).unwrap();
    let pdf = Pdf::new(Arc::new(bytes)).unwrap();
    pages
        .into_iter()
        .map(move |p| (p, extract_text_by_page(&pdf, p, &icon_map).unwrap()))
    // for page in pages {
    //     // let file_name = out_dir.join(format!("{page}.txt"));
    //     // let out = extract_text_by_page(&pdf, page, &icon_map).unwrap();
    //     // fs::write(file_name, out).unwrap();
    //     let out = fs::read_to_string(file_name).unwrap();
    // }
}
