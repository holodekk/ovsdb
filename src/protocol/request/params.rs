use std::fmt;

use serde::Serialize;

use super::Value;

#[derive(Serialize)]
pub struct Params(Vec<Value>);

impl Params {
    pub fn new<T, I>(args: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: Into<Value>,
    {
        let values = args.into_iter().map(|a| a.into()).collect();
        Self(values)
    }
}

impl Default for Params {
    fn default() -> Self {
        Self(vec![])
    }
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (idx, item) in self.0.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

impl Params {
    pub fn from<T, I>(args: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: Into<Value>,
    {
        let params = args.into_iter().map(|a| a.into()).collect();
        Self(params)
    }

    pub fn from_str<S>(arg: S) -> Self
    where
        S: AsRef<str> + Into<Value>,
    {
        Self(vec![arg.into()])
    }
}
