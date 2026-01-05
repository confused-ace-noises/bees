#[allow(clippy::module_inception)]
pub mod context;
pub mod pre_context;

use std::sync::LazyLock;

use context::{Context, CONTEXT};
use pre_context::PRE_CONTEXT;
use tokio::sync::RwLock;

use crate::context::pre_context::PreContext;

unsafe fn init_pre_context(lock: &mut Context) {
    let pre_context = &mut PRE_CONTEXT.blocking_write().0;
            
    pre_context.sort_unstable_by(|a, b| a.priority.number().cmp(&b.priority.number()));
    pre_context.iter()
        .for_each(|f| (f.func)(lock))
}

/// note: this must  be called *AFTER* `net::init_rate_limiter_duration()`
pub fn init_context() {
    let mut context = Context::new();

    unsafe { init_pre_context(&mut context) }

    CONTEXT
        .set(context)
        .expect("CONTEXT was already set when calling `init_context()`");
}

/// note: this must  be called *AFTER* `net::init_rate_limiter_duration()`
pub(crate) fn init_context_if_needed() {
    if CONTEXT.get().is_none() {
        let mut context = Context::new();

        unsafe { init_pre_context(&mut context) }

        let _ = CONTEXT.set(context);
    }
}

pub fn context() -> &'static Context {
    CONTEXT
        .get()
        .expect("this shouldn't happen. did you remember to init bees (`bees::init()`)?")
}

pub fn pre_context() -> &'static LazyLock<RwLock<PreContext>> {
    &pre_context::PRE_CONTEXT
}
