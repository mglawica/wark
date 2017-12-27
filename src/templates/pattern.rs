use std::fmt;
use trimmer::{Template, Options, Output, DataError, Variable};
use trimmer::{Context, RenderError};
use serde::de::{self, Visitor, Deserializer, Deserialize};

use templates::PARSER;


#[derive(Debug)]
pub struct Pattern {
    text: String,
    ast: Template,
}

struct PatternVisitor;


impl Pattern {
    pub fn render(&self, ctx: &Context) -> Result<String, RenderError> {
        self.ast.render(ctx)
    }
}

impl<'de> Visitor<'de> for PatternVisitor {
    type Value = Pattern;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid trimmer template")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        let options = Options::new()
            // TODO(tailhook) add validators
            .syntax_oneline()
            .clone();
        match PARSER.parse_with_options(&options, v) {
            Ok(tpl) => {
                Ok(Pattern {
                    text: v.to_string(),
                    ast: tpl,
                })
            }
            Err(e) => {
                Err(E::custom(e))
            }
        }
    }
}

impl<'a> Deserialize<'a> for Pattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'a>
    {
        deserializer.deserialize_str(PatternVisitor)
    }
}

impl<'render> Variable<'render> for Pattern {
    fn typename(&self) -> &'static str {
        "Pattern"
    }

    fn output(&self) -> Result<Output, DataError> {
        Ok((&self.text).into())
    }
}
