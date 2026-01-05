use std::sync::LazyLock;

use tokio::sync::RwLock;

use super::Context;

pub(crate) static PRE_CONTEXT: LazyLock<RwLock<PreContext>> = LazyLock::new(|| RwLock::new(PreContext::new()));

/// # SAFETY
/// this will do nothing if not called before 
/// initializing the Context.
pub unsafe fn push_pre_context<F>(value: F) 
where
    ContextMod: From<F>,
{
    PRE_CONTEXT.blocking_write().0.push(value.into());
}

pub struct PreContext(pub(crate) Vec<ContextMod>);

impl PreContext {
    pub const fn new() -> Self {
        Self(Vec::new())
    }
}

impl Default for PreContext {
    fn default() -> Self {
        Self::new()
    }
}

type ContextModFunc = Box<dyn Fn(&mut Context) + Send + Sync>;

pub struct ContextMod {
    pub priority: ContextModPriority,
    pub func: ContextModFunc,
}

impl ContextMod {
    pub fn new(priority: ContextModPriority, func: ContextModFunc ) -> Self {
        Self {
            priority,
            func,
        }
    }
}

pub enum ContextModPriority {
    Endpoint,
    Record,
    Resource,
    ClientMod,
    Custom(isize),
}

impl ContextModPriority {
    pub const CLIENT_MOD_NUMBER: isize = -4;
    pub const RECORD_NUMBER: isize = -3;
    pub const ENDPOINT_NUMBER: isize = -2;
    pub const RESOURCE_NUMBER: isize = -1;

    pub fn number(&self) -> isize {
        match self {
            ContextModPriority::Endpoint => Self::ENDPOINT_NUMBER,
            ContextModPriority::Record => Self::RECORD_NUMBER,
            ContextModPriority::Resource => Self::RESOURCE_NUMBER,
            ContextModPriority::ClientMod => Self::CLIENT_MOD_NUMBER,
            ContextModPriority::Custom(n) => *n,
        }
    }
}

impl From<ContextModPriority> for isize {
    fn from(value: ContextModPriority) -> Self {
        value.number()
    }
}