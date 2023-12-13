use crate::ecs::{components::TransformComponent, registry::Registry};
use anyhow::Result;

use super::SystemMaskBuilder;

pub struct MovementSystem;

impl MovementSystem {
    pub fn update(register: &mut Registry, dt: f64) -> Result<()> {
        //for entity in register.get_system_entities::<MovementSystem>()? {
            //let mut tf = register.get_component_mut::<TransformComponent>(*entity)?;
            //tf.position.0 += 20. * dt;
        //}
        Ok(())
    }

    pub fn get_system_mask() -> u32 {
        0
    }
}
