use std::mem;

use anyhow::{Context, anyhow};
#[cfg(feature = "bin-web")]
use gloo::console::log;
use itertools::Itertools;
use pest3_vm::Vm;
use pest3_vm::pest2::Token;
use serde_json::{Map, Value, json};

use crate::Beast;

pub struct Parser(Vm);

impl Parser {
    pub fn new(s: &str) -> anyhow::Result<Parser> {
        Ok(Self(
            Vm::from_src(s, "")
                .map_err(anyhow::Error::from_boxed)
                .context("parsing the grammar:")?,
        ))
    }

    pub fn parse_page(&self, s: &str) -> anyhow::Result<Vec<Value>> {
        log!("hello 1");
        let mut result = vec![];
        let mut token_stack = vec![];
        let parse = self.0.parse("main", s);
        log!("hello 2");
        for token in parse.map_err(|e| anyhow!("{e}"))?.flatten().tokens() {
            match token {
                Token::Start { rule, pos } => {
                    token_stack.push((rule, pos.pos(), Map::new()));
                }
                Token::End { rule, pos } => {
                    let (srule, start_pos, mut values) = token_stack
                        .pop()
                        .expect("there should always be a `Start` for an `End`");
                    assert!(rule == srule);
                    let mut value = json!(&s[start_pos..pos.pos()]);
                    if !values.is_empty() {
                        values.insert("_".to_owned(), value);
                        value = values.into();
                    }
                    if !token_stack.is_empty() {
                        let values = &mut token_stack.last_mut().unwrap().2;
                        if values.contains_key(rule) {
                            let entry = values.get_mut(rule).unwrap();
                            if let Some(entry) = entry.as_array_mut() {
                                entry.push(value);
                            } else {
                                let entry_value = mem::replace(entry, json!([]));
                                let entry = entry.as_array_mut().unwrap();
                                entry.push(entry_value);
                                entry.push(value);
                            }
                        } else {
                            values.insert(rule.to_owned(), value);
                        }
                    } else {
                        result.push(value);
                    }
                }
            }
            // result.insert(rule.to_owned(), json!(format!("{:?}",
            // pair.tokens()))); pair.tokens();
            // #[cfg(feature = "bin-web")]
            // log!(rule, format!("{value:?}"));
        }
        // todo!()
        // assert!(stack.len() == 1);
        Ok(result.into_iter().map_into().collect())
    }
}

pub fn parse_page_old(s: &str) -> Vec<Beast> {
    let vm = Vm::from_src(
        r#"
        ~ = _{ " " | "\t" | "\xA0" | '\u{2000}'..'\u{200A}' | "\u{202F}" | "\u{205F}" | "\u{3000}" }
        "#,
        "",
    );

    todo!("no");
}

pub fn parse_page(s: &str, grammar: &str) -> Vec<Beast> {
    let vm = Parser::new(grammar).unwrap();
    vm.parse_page(s).unwrap();
    todo!("HATH");
}
