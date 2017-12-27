use trimmer::{Variable, Output, Var, DataError};
use capturing_glob::Entry;

#[derive(Debug)]
pub struct GlobVar {
    path: String,
    captures: Vec<String>,
}

impl<'a> From<&'a Entry> for GlobVar {
    fn from(e: &Entry) -> GlobVar {
        let path = e.path().to_str().expect("path is utf8").to_string();
        let mut captures = Vec::new();
        for i in 0..10 {
            match e.group(i) {
                Some(x) => {
                    captures.push(x.to_str().expect("path is utf8")
                        .to_string());
                }
                None => break,
            }
        }
        return GlobVar { path, captures }
    }
}

impl<'render> Variable<'render> for GlobVar {
    fn typename(&self) -> &'static str {
        "GlobVar"
    }

    fn output(&self) -> Result<Output, DataError> {
        Ok((&self.path).into())
    }

    fn index<'x>(&'x self, key: &(Variable<'render> + 'render))
        -> Result<Var<'x, 'render>, DataError>
        where 'render: 'x
    {
        let index = key.as_int_key()?;
        match self.captures.get(index) {
            Some(x) => Ok(Var::borrow(x)),
            None => Ok(Var::undefined()),
        }
    }
}
