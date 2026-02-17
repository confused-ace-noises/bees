use std::{ops::{Deref, DerefMut}, sync::Arc};

use dashmap::DashSet;
use super::{resource::Resource, dyn_resource::DynResource};

#[derive(Debug, Default)]
pub struct ResourceManager(DashSet<DynResource>);

impl ResourceManager {
    pub fn new() -> Self {
        Self(DashSet::new())
    }

    #[inline]
    pub fn add_dyn_resource(&self, resource: Arc<dyn Resource>) -> bool {
        self.0.insert(DynResource(resource))
    }
    
    #[inline]
    pub fn add_resource<T: Resource + 'static>(&self, resource: T) -> bool {
        self.0.insert(DynResource::from_res(resource))
    }

    // #[inline]
    // pub fn get_resource_ref(&self, ident: &str) -> Option<dashmap::setref::one::Ref<'_, Box<dyn Resource>>> {
    //     self.0.get(ident)
    // }

    // // #[inline]
    // // pub fn get_resource(&self, ident: &str) -> Option<DynResource> {
    // //     self.0.get_resource_ref(ident).map(|x| x.clone())
    // // }

    // #[inline]
    // pub fn remove_resource(&self, ident: &str) -> Option<Box<dyn Resource>> {
    //     self.0.remove(ident) 
    // }

    // #[inline]
    // pub fn remove_resource_if(&self, ident: &str, f: impl FnOnce(&Box<dyn Resource>) -> bool) -> Option<Box<dyn Resource>> {
    //     self.0.remove_if(ident, f)
    // }

    // #[inline]
    // pub fn contains_resource(&self, ident: &str) -> bool {
    //     self.0.contains(ident)
    // }
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