use crate::ecs::{components::TransformComponent, registry::Registry};
use anyhow::Result;

pub struct RenderSystem;

impl RenderSystem {
    pub fn update(register: &mut Registry, dt: f64) -> Result<()> {
        for entity in register.get_system_entities::<RenderSystem>()? {
            //let mut tf = register.get_component_mut::<TransformComponent>(*entity)?;
        }
        Ok(())
    }

    pub fn get_system_mask() -> u32 {
        0
    }
}
