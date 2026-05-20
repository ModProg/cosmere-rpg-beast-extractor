use anyhow::Context;
use pest3_vm::Vm;

use crate::Beast;

struct Parser(Vm);

impl Parser {
    fn new(s: &str) -> anyhow::Result<Parser> {
        Ok(Self(
            Vm::from_src(s, "")
                .map_err(anyhow::Error::from_boxed)
                .context("parsing the grammar")?,
        ))
    }

    fn parse_page(&self, s: &str) -> anyhow::Result<Vec<serde_json::Value>> {
        for pair in self.0.parse("main", s)? {
            println!("{}", pair.as_rule());
        }
        panic!()
    }
}

pub fn parse_page(s: &str) -> Vec<Beast> {
    let vm = Vm::from_src(r#""#, "");

    todo!();
}
