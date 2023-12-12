use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use anyhow::Result;

use crate::logger::Logger;

use super::ecs_errors::EcsErrors;

pub type ComponentMap = HashMap<TypeId, Vec<Option<Rc<RefCell<dyn Any>>>>>;

const MAX_COMPONENTS: usize = 32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

impl Entity {
    pub fn new(num: u32) -> Self {
        Self(num)
    }
}

#[derive(Default)]
pub struct Registry {
    logger: Rc<RefCell<Logger>>,
    entity_count: u32,
    /// key => ComponentTypeId, value => Vec<Component>
    components: ComponentMap,
    /// index: component id => component mask
    component_signatures: HashMap<TypeId, u32>,
    /// index: entity_id => signature mask
    entity_component_signatures: Vec<u32>,
    entities_to_be_added: HashSet<Entity>,
    entities_to_be_killed: HashSet<Entity>,
    // NOTE: push_back, pop_front (remove later)
    available_entity_spots: VecDeque<Entity>,
}

impl Registry {
    pub fn new(logger: Rc<RefCell<Logger>>) -> Self {
        Self {
            logger,
            ..Default::default()
        }
    }

    pub fn register_component<T: Any + 'static>(&mut self) -> Result<()> {
        if self.components.len() >= MAX_COMPONENTS {
            return Err(EcsErrors::MaxComponentReached.into());
        }
        let type_id = TypeId::of::<T>();
        self.components.insert(type_id, vec![]);
        self.component_signatures
            .insert(type_id, 1 << self.component_signatures.len());
        Ok(())
    }

    pub fn create_entity(&mut self) -> Result<()> {
        if self.available_entity_spots.is_empty() {
            // create entity
            let entity = Entity::new(self.entity_count);
            self.entity_count += 1;
            self.entities_to_be_added.insert(entity);

            Ok(())
        } else {
            todo!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Health(i32);
    struct Size(i32);

    #[test]
    fn creating_components() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<Health>()?;
        registry.register_component::<Size>()?;
        assert_eq!(registry.components.len(), 2);
        assert_eq!(registry.component_signatures.len(), 2);
        Ok(())
    }

    #[test]
    fn check_max_components() -> Result<()> {
        let mut registry = Registry::default();
        struct Type1;
        struct Type2;
        struct Type3;
        struct Type4;
        struct Type5;
        struct Type6;
        struct Type7;
        struct Type8;
        struct Type9;
        struct Type10;
        struct Type11;
        struct Type12;
        struct Type13;
        struct Type14;
        struct Type15;
        struct Type16;
        struct Type17;
        struct Type18;
        struct Type19;
        struct Type20;
        struct Type21;
        struct Type22;
        struct Type23;
        struct Type24;
        struct Type25;
        struct Type26;
        struct Type27;
        struct Type28;
        struct Type29;
        struct Type30;
        struct Type31;
        struct Type32;
        struct Type33;
        registry.register_component::<Type1>()?;
        registry.register_component::<Type2>()?;
        registry.register_component::<Type3>()?;
        registry.register_component::<Type4>()?;
        registry.register_component::<Type5>()?;
        registry.register_component::<Type6>()?;
        registry.register_component::<Type7>()?;
        registry.register_component::<Type8>()?;
        registry.register_component::<Type9>()?;
        registry.register_component::<Type10>()?;
        registry.register_component::<Type11>()?;
        registry.register_component::<Type12>()?;
        registry.register_component::<Type13>()?;
        registry.register_component::<Type14>()?;
        registry.register_component::<Type15>()?;
        registry.register_component::<Type16>()?;
        registry.register_component::<Type17>()?;
        registry.register_component::<Type18>()?;
        registry.register_component::<Type19>()?;
        registry.register_component::<Type20>()?;
        registry.register_component::<Type21>()?;
        registry.register_component::<Type22>()?;
        registry.register_component::<Type23>()?;
        registry.register_component::<Type24>()?;
        registry.register_component::<Type25>()?;
        registry.register_component::<Type26>()?;
        registry.register_component::<Type27>()?;
        registry.register_component::<Type28>()?;
        registry.register_component::<Type29>()?;
        registry.register_component::<Type30>()?;
        registry.register_component::<Type31>()?;
        registry.register_component::<Type32>()?;
        let result = registry.register_component::<Type33>();
        assert_eq!(result.is_err(), true);
        Ok(())
    }
}
