use std::sync::Arc;

use crate::capability::Capability;

pub trait Record: Send {
    const SHARED_URL: &str;
    fn shared_caps() -> Arc<[Box<dyn Capability>]>;
}