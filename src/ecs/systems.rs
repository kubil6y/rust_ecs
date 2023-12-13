use super::{ecs_errors::EcsErrors, registry::Registry};
use anyhow::Result;
use std::any::Any;

pub mod movement_system;
pub mod render_system;

// NOTE: comeback to this
pub struct SystemMaskBuilder<'a> {
    mask: u32,
    registry: &'a Registry,
}

// NOTE: comeback to this
impl<'a> SystemMaskBuilder<'a> {
    pub fn new(registry: &'a Registry) -> Self {
        Self { mask: 0, registry }
    }

    pub fn with<T: Any>(&mut self) -> Result<&mut Self> {
        let component_mask = self
            .registry
            .get_component_mask::<T>()
            .ok_or(EcsErrors::ComponentDoesNotExist)?;
        self.mask |= component_mask;
        Ok(self)
    }

    pub fn build(&self) -> u32 {
        self.mask
    }
}
