use semver::Version;

use quire::ast::{Ast, Tag};
use quire::{Error as QuireError, ErrorCollector};
use quire::validate as V;


pub struct MinimumVersion;

impl V::Validator for MinimumVersion {
    fn default(&self, _pos: V::Pos) -> Option<Ast> {
        None
    }
    fn validate(&self, ast: Ast, err: &ErrorCollector) -> Ast {
        let (pos, kind, val) = match ast {
            Ast::Scalar(pos, _, kind, min_version) => {
                let cur = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
                match Version::parse(&min_version) {
                    Ok(ref ver) if ver > &cur => {
                        err.add_error(QuireError::validation_error(
                            &pos,
                            format!(
                                "Please upgrade wark to at least {:?}",
                                min_version)));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        err.add_error(QuireError::custom_at(&pos, e));
                    }
                }
                (pos, kind, min_version)
            }
            Ast::Null(_, _, _) => {
                return ast;
            }
            ast => {
                err.add_error(QuireError::validation_error(&ast.pos(),
                    format!("Value must be scalar")));
                return ast;
            }
        };
        Ast::Scalar(pos, Tag::NonSpecific, kind, val)
    }
}
