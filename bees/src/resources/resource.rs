use std::{borrow::Borrow, fmt::Debug, hash::Hash};

use crate::ResourceOutput;

pub trait Resource: Debug + Send + Sync {
    fn ident(&self) -> &str;
    fn data<'a>(&'a self) -> ResourceOutput<'a>;
}

impl PartialEq for dyn Resource {
    fn eq(&self, other: &Self) -> bool {
        self.ident() == other.ident()
    }
}
impl Eq for dyn Resource {}
impl Hash for dyn Resource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident().hash(state);
    }
}

impl Borrow<str> for dyn Resource {
    fn borrow(&self) -> &str {
        self.ident()
    }
}

impl Borrow<str> for Box<dyn Resource> {
    fn borrow(&self) -> &str {
        self.ident()
    }
}