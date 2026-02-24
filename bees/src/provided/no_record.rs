use std::sync::Arc;

use crate::record::Record;

pub struct NoRecord;
impl Record for NoRecord {
    const SHARED_URL: &str = "";

    fn shared_caps() -> std::sync::Arc<[Box<dyn crate::capability::Capability>]> {
        Arc::new([])
    }
}