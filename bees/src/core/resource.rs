#![allow(unused_parens)]
use std::{borrow::Borrow, fmt::{Debug, Display}, hash::Hash, ops::{Deref, DerefMut}, sync::Arc};
use async_trait::async_trait;
use dashmap::DashSet;


/// # Resource
/// `Resource` represents a commonly needed piece of data, like
/// a cookie or a UUID.
/// 
/// The `ident` field should be as inexpensive to run as possible,
/// because it's used internally as a means of indexing; this also
/// means that it's bad practice to make any two `Resource`s have
/// implementations that might clash.
/// 
/// # Usage
/// 
/// ```
/// # use async_trait::async_trait;
/// pub struct Cookie {
///     cookie_name: String,
///     cookie_string: String,
/// }
/// 
/// impl Cookie {
///     // note: use interior mutability if needed
///     pub fn update_cookie(&self) {
///         // expensive updating logic here...
///     } 
/// }
/// 
/// #[async_trait]
/// impl Resource for Cookie {
///     // calling ident is cheap
///     fn ident(&self) -> &str {
///         self.cookie_name.as_str()
///     }
///     
///     // calling data can be expensive
///     async fn data(&self, client: Client) -> String {
///         self.update_cookie();
///         self.cookie_string
///     }
/// }
/// ```
#[async_trait]
pub trait Resource: Send + Sync + Debug {
    fn ident(&self) -> &str;
    async fn data(&self) -> Box<dyn Display>;
}

impl PartialEq for (dyn Resource + 'static) {
    fn eq(&self, other: &Self) -> bool {
        self.ident() == other.ident()
    }
}
impl Eq for (dyn Resource + 'static) {}

impl Hash for (dyn Resource + 'static) {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident().hash(state);
    }
}


impl PartialEq for DynResource {
    fn eq(&self, other: &Self) -> bool {
        self.ident() == other.ident()
    }
}

impl Eq for DynResource {}

impl Hash for DynResource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.ident().hash(state);
    }
}

#[derive(Default, Debug)]
pub struct ResourceManager {
    resources: Arc<DashSet<DynResource>>,
}

#[derive(Debug)]
pub struct DynResource(Arc<dyn Resource>);

impl DynResource {
    pub fn from_res<R: Resource + 'static>(resource: R) -> Self {
        DynResource(Arc::new(resource) as Arc<dyn Resource>)
    }
}

impl<R> From<Arc<R>> for DynResource 
where
    R: Resource + 'static,
{
    fn from(value: Arc<R>) -> Self {
        DynResource(value)
    }
}

impl Clone for DynResource {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[async_trait]
impl Resource for DynResource {
    fn ident(&self) -> &str {
        self.0.ident()
    }

    async fn data(&self) -> Box<dyn Display> {
        Box::new(self.0.data().await)
    }
}

impl Deref for DynResource {
    type Target = Arc<dyn Resource>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DynResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ResourceManager {
    pub async fn new() -> Self {
        Self {
            resources: Arc::new(DashSet::new()),
        }
    }

    #[inline]
    pub fn add_dyn_resource(&self, resource: DynResource) -> bool {
        self.resources.insert(resource)
    }
    
    #[inline]
    pub fn add_resource<T: Resource + 'static>(&self, resource: T) -> bool {
        self.resources.insert(DynResource::from_res(resource))
    }

    #[inline]
    pub fn get_resource_ref(&self, ident: &str) -> Option<dashmap::setref::one::Ref<'_, DynResource>> {
        self.resources.get(ident)
    }

    #[inline]
    pub fn get_resource(&self, ident: &str) -> Option<DynResource> {
        self.get_resource_ref(ident).and_then(|x| Some(x.clone()))
    }

    #[inline]
    pub fn remove_resource(&self, ident: &str) -> Option<DynResource> {
        self.resources.remove(ident) 
    }

    #[inline]
    pub fn remove_resource_if(&self, ident: &str, f: impl FnOnce(&DynResource) -> bool) -> Option<DynResource> {
        self.resources.remove_if(ident, f)
    }

    #[inline]
    pub fn contains_resource(&self, ident: &str) -> bool {
        self.resources.contains(ident)
    }
}

impl Clone for ResourceManager {
    fn clone(&self) -> Self {
        Self { resources: Arc::clone(&self.resources) }
    }
}


impl Borrow<str> for (dyn Resource + 'static) {
    #[inline]
    fn borrow(&self) -> &str {
        self.ident()
    }
}

impl Borrow<str> for DynResource {
    fn borrow(&self) -> &str {
        self.ident()
    }
}