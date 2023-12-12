use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use anyhow::Result;

use crate::logger::Logger;

use super::ecs_errors::EcsErrors;

pub type Component = Rc<RefCell<dyn Any>>;
pub type Components = HashMap<TypeId, Vec<Option<Component>>>;

pub const MAX_COMPONENTS: usize = 32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

impl Entity {
    pub fn new(num: usize) -> Self {
        Self(num)
    }
}

#[derive(Default)]
pub struct Registry {
    logger: Rc<RefCell<Logger>>,
    num_entities: usize,
    /// key => ComponentTypeId, value => Vec<Component>
    components: HashMap<TypeId, Vec<Option<Component>>>,
    //component_pools: Vec<Rc<RefCell<Vec<Rc<RefCell<dyn Any>>>>>>,
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
        let components_length = self.components.len();

        self.components.insert(type_id, vec![]);

        self.component_signatures
            .insert(type_id, 1 << components_length);

        Ok(())
    }

    pub fn create_entity(&mut self) -> Entity {
        if self.available_entity_spots.is_empty() {
            let entity = Entity::new(self.num_entities);
            self.num_entities += 1;
            self.entities_to_be_added.insert(entity);

            // Fill component for all the component types None by default
            for (_type_id, components_vec) in self.components.iter_mut() {
                components_vec.push(None);
            }

            self.entity_component_signatures.push(0);

            self.logger
                .borrow_mut()
                .log(&format!("Entity created with id = {}", entity.0));

            entity
        } else {
            todo!();
        }
    }

    // Component management
    pub fn add_component(&mut self, entity: Entity, data: impl Any) -> Result<()> {
        let entity_id = entity.0;
        let component_type_id = data.type_id();

        let component_vec = match self.components.get_mut(&component_type_id) {
            Some(component_vec) => component_vec,
            None => return Err(EcsErrors::ComponentDoesNotExist.into()),
        };

        component_vec[entity_id] = Some(Rc::new(RefCell::new(data)));

        let component_mask = match self.get_component_signature_by_id(component_type_id) {
            Some(mask) => mask,
            None => return Err(EcsErrors::ComponentMaskDoesNotExist.into()),
        };

        if let Some(entity_mask) = self.entity_component_signatures.get_mut(entity_id) {
            *entity_mask |= component_mask;
        } else {
            return Err(EcsErrors::EntityComponentMaskDoesNotExist.into());
        }

        Ok(())
    }

    pub fn remove_component<T: Any>(&mut self, entity: Entity) -> Result<bool> {
        if self.has_component::<T>(entity)? {
            let component_mask = self
                .get_component_signature_by_type::<T>()
                .ok_or(EcsErrors::ComponentMaskDoesNotExist)?;

            let entity_mask = self
                .entity_component_signatures
                .get_mut(entity.0)
                .ok_or(EcsErrors::EntityComponentMaskDoesNotExist)?;

            *entity_mask ^= component_mask;

            self.logger.borrow_mut().log(&format!(
                "Component id = {:?} was removed from entity id {}",
                &TypeId::of::<T>(),
                entity.0
            ));
            return Ok(true);
        }
        Ok(false)
    }

    pub fn has_component<T: Any>(&self, entity: Entity) -> Result<bool> {
        let component_mask = self
            .get_component_signature_by_type::<T>()
            .ok_or(EcsErrors::ComponentMaskDoesNotExist)?;

        let entity_mask = self
            .get_entity_component_signature(entity)
            .ok_or(EcsErrors::EntityComponentMaskDoesNotExist)?;

        return Ok(entity_mask & component_mask == component_mask);
    }

    pub fn get_component_ref<T: Any>(&self, entity: Entity) -> Ref<T> {
        todo!();
    }

    pub fn get_component_mut<T: Any>(&self, entity: Entity) -> RefMut<T> {
        todo!();
    }

    pub fn get_num_entities(&self) -> usize {
        self.num_entities
    }

    pub fn get_entity_component_signature(&self, entity: Entity) -> Option<u32> {
        self.entity_component_signatures.get(entity.0).copied()
    }

    pub fn get_component_signature_by_id(&self, type_id: TypeId) -> Option<u32> {
        self.component_signatures.get(&type_id).copied()
    }

    pub fn get_component_signature_by_type<T: Any>(&self) -> Option<u32> {
        self.component_signatures.get(&TypeId::of::<T>()).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Health(i32);
    struct Size(i32);

    #[test]
    fn removing_components_from_entities() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<Health>()?;
        registry.register_component::<Size>()?;
        let entity1 = registry.create_entity();
        let entity2 = registry.create_entity();
        registry.add_component(entity1, Health(100))?;
        registry.add_component(entity1, Size(25))?;
        registry.add_component(entity2, Size(30))?;

        let health_mask = registry
            .get_component_signature_by_type::<Health>()
            .unwrap();
        let size_mask = registry.get_component_signature_by_type::<Size>().unwrap();

        assert_eq!(
            health_mask | size_mask,
            registry.get_entity_component_signature(entity1).unwrap()
        );

        registry.remove_component::<Health>(entity1)?;

        assert_eq!(
            size_mask,
            registry.get_entity_component_signature(entity1).unwrap()
        );

        registry.remove_component::<Size>(entity1)?;
        registry.remove_component::<Size>(entity2)?;

        assert_eq!(0, registry.get_entity_component_signature(entity1).unwrap());
        assert_eq!(0, registry.get_entity_component_signature(entity2).unwrap());

        Ok(())
    }

    #[test]
    fn adding_component_to_entities() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<Health>()?;
        registry.register_component::<Size>()?;
        let entity1 = registry.create_entity();
        let entity2 = registry.create_entity();

        registry.add_component(entity1, Health(100))?;
        registry.add_component(entity1, Size(25))?;
        registry.add_component(entity2, Size(30))?;

        // Testing component values
        let health_type_id = TypeId::of::<Health>();
        let size_type_id = TypeId::of::<Size>();

        let health_vec = registry.components.get(&health_type_id).unwrap();
        let size_vec = registry.components.get(&size_type_id).unwrap();

        let wrapped_entity1_health = health_vec[entity1.0].as_ref().unwrap();
        let borrowed_entity1_health = wrapped_entity1_health.as_ref().borrow();
        let entity1_health = borrowed_entity1_health.downcast_ref::<Health>().unwrap();
        assert_eq!(100, entity1_health.0);

        let wrapped_entity1_size = size_vec[entity1.0].as_ref().unwrap();
        let borrowed_entity1_size = wrapped_entity1_size.as_ref().borrow();
        let entity1_size = borrowed_entity1_size.downcast_ref::<Size>().unwrap();
        assert_eq!(25, entity1_size.0);

        let wrapped_entity2_size = size_vec[entity2.0].as_ref().unwrap();
        let borrowed_entity2_size = wrapped_entity2_size.as_ref().borrow();
        let entity2_size = borrowed_entity2_size.downcast_ref::<Size>().unwrap();
        assert_eq!(30, entity2_size.0);

        // Testing entity component masks
        let health_mask = registry
            .get_component_signature_by_type::<Health>()
            .unwrap();
        let size_mask = registry.get_component_signature_by_type::<Size>().unwrap();
        let expected_entity1_mask = health_mask | size_mask;
        let expected_entity2_mask = size_mask;

        assert_eq!(
            expected_entity1_mask,
            registry.get_entity_component_signature(entity1).unwrap()
        );
        assert_eq!(
            expected_entity2_mask,
            registry.get_entity_component_signature(entity2).unwrap()
        );

        Ok(())
    }

    #[test]
    fn creating_components() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<Health>()?;
        registry.register_component::<Size>()?;
        assert_eq!(registry.components.len(), 2);
        assert_eq!(registry.component_signatures.len(), 2);
        let health_signature_expected = 1 << 0;
        let size_signature_expected = 1 << 1;
        assert_eq!(
            health_signature_expected,
            registry
                .get_component_signature_by_type::<Health>()
                .unwrap()
        );
        assert_eq!(
            size_signature_expected,
            registry.get_component_signature_by_type::<Size>().unwrap()
        );
        Ok(())
    }

    #[test]
    fn create_entities() {
        let mut registry = Registry::default();
        let entity1 = registry.create_entity();
        let entity2 = registry.create_entity();

        assert_eq!(entity1.0, 0);
        assert_eq!(entity2.0, 1);
        assert_eq!(registry.entities_to_be_added.len(), 2);
        assert_eq!(registry.entities_to_be_added.contains(&entity1), true);
        assert_eq!(registry.entities_to_be_added.contains(&entity2), true);
        assert_eq!(registry.get_num_entities(), 2);

        // testing if components_vec is filled with default None values (nullptr)
        for (type_id, _) in registry.components.iter() {
            let components_vec = registry.components.get(type_id).unwrap();
            assert_eq!(components_vec.len(), 2);
            for el in components_vec.iter() {
                assert_eq!(el.is_none(), true);
            }
        }
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

        let type33 = registry.register_component::<Type33>();
        assert_eq!(type33.is_err(), true);
        Ok(())
    }
}
