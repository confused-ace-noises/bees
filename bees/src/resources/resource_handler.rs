use std::ops::{Deref, DerefMut};

use dashmap::{DashSet, setref::one::Ref};

use super::{resource::Resource, dyn_resource::DynResource};

#[derive(Debug, Default)]
pub struct ResourceManager(DashSet<DynResource>);

impl ResourceManager {
    pub fn new() -> Self {
        Self(DashSet::new())
    }
    
    #[inline]
    pub fn add_resource<T: Resource + 'static>(&self, resource: T) -> bool {
        self.0.insert(DynResource::from_res(resource))
    }

    #[inline]
    pub fn get_resource<T: AsRef<str>>(&self, ident: T) -> Option<Ref<'_, DynResource>>{
        self.get(ident.as_ref())
    }
}

impl Deref for ResourceManager {
    type Target = DashSet<DynResource>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ResourceManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}