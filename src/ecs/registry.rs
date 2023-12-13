use super::ecs_errors::EcsErrors;
use crate::logger::Logger;
use anyhow::Result;
use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

pub const MAX_COMPONENTS: usize = 32;
pub type Component = Rc<RefCell<dyn Any>>;

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
    component_masks: HashMap<TypeId, u32>,
    /// index: entity_id => signature mask
    entity_masks: Vec<u32>,

    // TODO: no functionality yet
    entities_to_be_added: Vec<Entity>,
    entities_to_be_killed: Vec<Entity>,
    // NOTE: push_back, pop_front (should be Rc<RefCell<VecDeque<Entity>>> as well)
    available_entity_spots: VecDeque<Entity>,

    system_masks: Rc<RefCell<HashMap<TypeId, u32>>>,
    system_entities: Rc<RefCell<HashMap<TypeId, HashSet<Entity>>>>,
}

impl Registry {
    pub fn new(logger: Rc<RefCell<Logger>>) -> Self {
        Self {
            logger,
            ..Default::default()
        }
    }

    pub fn update(&mut self) -> Result<()> {
        // HOW HARD IT CAN BE TO DO A FUCKING NESTED loop in a struct
        // leftoff here for future refence...

        for (system_type_id, system_mask) in self.system_masks.borrow_mut().iter_mut() {
            for entity in &self.entities_to_be_added {
                let entity_mask = self.get_entity_mask_mut(*entity).unwrap();

                if *entity_mask & *system_mask == *system_mask {
                    self.add_entity_to_system_with_id(system_type_id, *entity);
                }
            }
        }
        //let to_be_added = self.entities_to_be_added.iter_mut();
        //for entity in to_be_added{
        //let entity_mask = self
        //.get_entity_mask(*entity)
        //.ok_or(EcsErrors::EntityComponentMaskDoesNotExist)?;
        //for (system_type_id, system_mask) in self.system_masks.borrow_mut().iter_mut() {
        //if entity_mask & *system_mask == *system_mask {
        //self.add_entity_to_system_with_id(system_type_id, *entity);
        //}
        //}
        //}

        self.entities_to_be_added.clear();

        Ok(())
    }

    pub fn register_component<T: Any + 'static>(&mut self) -> Result<()> {
        if self.components.len() >= MAX_COMPONENTS {
            return Err(EcsErrors::MaxComponentReached.into());
        }
        let type_id = TypeId::of::<T>();
        let components_length = self.components.len();
        self.components.insert(type_id, vec![]);
        self.component_masks.insert(type_id, 1 << components_length);
        Ok(())
    }

    pub fn create_entity(&mut self) -> Entity {
        if self.components.len() == 0 {
            panic!("Register components first and then create entities!");
        }
        if self.available_entity_spots.is_empty() {
            let entity = Entity::new(self.num_entities);
            self.num_entities += 1;

            if !self.entities_to_be_added.contains(&entity) {
                self.entities_to_be_added.push(entity);
            }

            // Fill component for all the component types None by default
            for (_type_id, components_vec) in self.components.iter_mut() {
                components_vec.push(None);
            }

            self.entity_masks.push(0);

            self.logger
                .as_ref()
                .borrow_mut()
                .log(&format!("Entity created with id = {}", entity.0));

            entity
        } else {
            todo!();
        }
    }

    // Component management
    /// NOTE: If you add the same component again it will override
    pub fn add_component(&mut self, entity: Entity, data: impl Any) -> Result<()> {
        let entity_id = entity.0;
        let component_type_id = data.type_id();

        let component_vec = self
            .components
            .get_mut(&component_type_id)
            .ok_or(EcsErrors::ComponentDoesNotExist)?;

        component_vec[entity_id] = Some(Rc::new(RefCell::new(data)));

        let component_mask = self
            .get_component_mask_with_id(component_type_id)
            .ok_or(EcsErrors::ComponentMaskDoesNotExist)?;

        if let Some(entity_mask) = self.entity_masks.get_mut(entity_id) {
            *entity_mask |= component_mask;
        } else {
            return Err(EcsErrors::EntityComponentMaskDoesNotExist.into());
        }

        Ok(())
    }

    pub fn remove_component<T: Any>(&mut self, entity: Entity) -> Result<bool> {
        let component_mask = self
            .get_component_mask::<T>()
            .ok_or(EcsErrors::ComponentMaskDoesNotExist)?;

        if self.has_component_with_mask(entity, component_mask)? {
            let entity_mask = self
                .get_entity_mask_mut(entity)
                .ok_or(EcsErrors::EntityComponentMaskDoesNotExist)?;

            *entity_mask ^= component_mask;

            self.logger.as_ref().borrow_mut().log(&format!(
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
            .get_component_mask::<T>()
            .ok_or(EcsErrors::ComponentMaskDoesNotExist)?;

        let entity_mask = self
            .get_entity_mask(entity)
            .ok_or(EcsErrors::EntityComponentMaskDoesNotExist)?;

        Ok(entity_mask & component_mask == component_mask)
    }

    pub fn has_component_with_mask(&self, entity: Entity, component_mask: u32) -> Result<bool> {
        let entity_mask = self
            .get_entity_mask(entity)
            .ok_or(EcsErrors::EntityComponentMaskDoesNotExist)?;
        Ok(entity_mask & component_mask == component_mask)
    }

    fn extract_components<T: Any>(&self) -> Result<&Vec<Option<Component>>> {
        let type_id = TypeId::of::<T>();
        let components = self
            .components
            .get(&type_id)
            .ok_or(EcsErrors::ComponentDoesNotExist)?;
        Ok(components)
    }

    pub fn get_component<T: Any>(&self, entity: Entity) -> Result<Ref<T>> {
        if !self.has_component::<T>(entity)? {
            return Err(EcsErrors::ComponentDoesNotExist.into());
        }
        let components = self.extract_components::<T>()?;
        let borrowed_component = components[entity.0]
            .as_ref()
            .ok_or(EcsErrors::ComponentDoesNotExist)?
            .borrow();
        Ok(Ref::map(borrowed_component, |any| {
            any.downcast_ref::<T>().unwrap()
        }))
    }

    pub fn get_component_mut<T: Any + 'static>(&self, entity: Entity) -> Result<RefMut<T>> {
        if !self.has_component::<T>(entity)? {
            return Err(EcsErrors::ComponentDoesNotExist.into());
        }
        let components = self.extract_components::<T>()?;
        let borrowed_component = components[entity.0]
            .as_ref()
            .ok_or(EcsErrors::ComponentDoesNotExist)?
            .borrow_mut();
        Ok(RefMut::map(borrowed_component, |any| {
            any.downcast_mut::<T>().unwrap()
        }))
    }

    pub fn get_num_entities(&self) -> usize {
        self.num_entities
    }

    pub fn get_num_systems(&self) -> usize {
        self.system_masks.borrow().len()
    }

    pub fn get_entity_mask(&self, entity: Entity) -> Option<u32> {
        self.entity_masks.get(entity.0).copied()
    }

    pub fn get_entity_mask_mut(&mut self, entity: Entity) -> Option<&mut u32> {
        self.entity_masks.get_mut(entity.0)
    }

    pub fn get_component_mask<T: Any>(&self) -> Option<u32> {
        self.component_masks.get(&TypeId::of::<T>()).copied()
    }

    pub fn get_component_mask_with_id(&self, type_id: TypeId) -> Option<u32> {
        self.component_masks.get(&type_id).copied()
    }

    pub fn register_system<T: Any>(&mut self, system_mask: u32) -> Result<bool> {
        let type_id = TypeId::of::<T>();
        if self.system_masks.borrow().contains_key(&type_id) {
            return Ok(false);
        }
        self.system_masks.borrow_mut().insert(type_id, system_mask); // this wrong
        self.system_entities
            .borrow_mut()
            .insert(type_id, HashSet::new());
        Ok(true)
    }

    pub fn get_system_entities<T: Any>(&self) -> Result<HashSet<Entity>> {
        let type_id = TypeId::of::<T>();
        let borrowed_entities = self.system_entities.borrow();
        let entities = borrowed_entities
            .get(&type_id)
            .ok_or(EcsErrors::SystemDoesNotExist)?;

        // making copies of this array on each iteration? what the fuck is this lmfao

        Ok(entities.clone())
    }

    pub fn add_entity_to_system<T: Any>(&mut self, entity: Entity) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let mut borrowed_entities = self.system_entities.borrow_mut();
        let entities = borrowed_entities
            .get_mut(&type_id)
            .ok_or(EcsErrors::SystemDoesNotExist)?;
        entities.insert(entity);
        Ok(())
    }

    pub fn add_entity_to_system_with_id(&mut self, type_id: &TypeId, entity: Entity) -> Result<()> {
        let mut borrowed_entities = self.system_entities.borrow_mut();
        let entities = borrowed_entities
            .get_mut(&type_id)
            .ok_or(EcsErrors::SystemDoesNotExist)?;
        entities.insert(entity);
        Ok(())
    }

    pub fn remove_entity_from_system<T: Any>(&mut self, entity: Entity) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let mut borrowed_entities = self.system_entities.borrow_mut();
        let entities = borrowed_entities
            .get_mut(&type_id)
            .ok_or(EcsErrors::SystemDoesNotExist)?;
        entities.remove(&entity);
        Ok(())
    }

    pub fn remove_entity_from_system_with_id(
        &mut self,
        type_id: &TypeId,
        entity: Entity,
    ) -> Result<()> {
        let mut borrowed_entities = self.system_entities.borrow_mut();
        let entities = borrowed_entities
            .get_mut(&type_id)
            .ok_or(EcsErrors::SystemDoesNotExist)?;
        entities.remove(&entity);
        Ok(())
    }

    pub fn get_system_mask<T: Any>(&self) -> Result<u32> {
        let mask = self
            .system_masks
            .borrow()
            .get(&TypeId::of::<T>())
            .copied()
            .ok_or(EcsErrors::SystemDoesNotExist)?;
        Ok(mask)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct Health(i32);
    #[derive(Debug)]
    struct Size(i32);

    #[test]
    fn getting_mut_components_from_entities() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<Health>()?;
        registry.register_component::<Size>()?;
        let entity1 = registry.create_entity();
        registry.add_component(entity1, Health(50))?;
        registry.add_component(entity1, Size(100))?;

        let mut entity1_health = registry.get_component_mut::<Health>(entity1)?;
        entity1_health.0 = 30;
        assert_eq!(entity1_health.0, 30);

        let mut entity1_size = registry.get_component_mut::<Size>(entity1)?;
        entity1_size.0 = 5;
        assert_eq!(entity1_size.0, 5);

        Ok(())
    }

    #[test]
    fn getting_components_from_entities() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<Health>()?;
        registry.register_component::<Size>()?;
        let entity1 = registry.create_entity();
        let entity2 = registry.create_entity();
        registry.add_component(entity1, Health(50))?;
        registry.add_component(entity2, Health(100))?;
        {
            let entity1_health = registry.get_component::<Health>(entity1)?;
            let entity2_health = registry.get_component::<Health>(entity2)?;
            assert_eq!(entity1_health.0, 50);
            assert_eq!(entity2_health.0, 100);
        }
        registry.remove_component::<Health>(entity1)?;
        let wrapped_entity1_health = registry.get_component::<Health>(entity1);
        assert_eq!(wrapped_entity1_health.is_err(), true);

        Ok(())
    }

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

        let health_mask = registry.get_component_mask::<Health>().unwrap();
        let size_mask = registry.get_component_mask::<Size>().unwrap();

        assert_eq!(
            health_mask | size_mask,
            registry.get_entity_mask(entity1).unwrap()
        );

        registry.remove_component::<Health>(entity1)?;

        assert_eq!(size_mask, registry.get_entity_mask(entity1).unwrap());

        registry.remove_component::<Size>(entity1)?;
        registry.remove_component::<Size>(entity2)?;

        assert_eq!(0, registry.get_entity_mask(entity1).unwrap());
        assert_eq!(0, registry.get_entity_mask(entity2).unwrap());

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
        let health_mask = registry.get_component_mask::<Health>().unwrap();
        let size_mask = registry.get_component_mask::<Size>().unwrap();
        let expected_entity1_mask = health_mask | size_mask;
        let expected_entity2_mask = size_mask;

        assert_eq!(
            expected_entity1_mask,
            registry.get_entity_mask(entity1).unwrap()
        );
        assert_eq!(
            expected_entity2_mask,
            registry.get_entity_mask(entity2).unwrap()
        );

        Ok(())
    }

    #[test]
    fn creating_components() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<Health>()?;
        registry.register_component::<Size>()?;
        assert_eq!(registry.components.len(), 2);
        assert_eq!(registry.component_masks.len(), 2);
        let health_mask_expected = 1 << 0;
        let size_mask_expected = 1 << 1;
        assert_eq!(
            health_mask_expected,
            registry.get_component_mask::<Health>().unwrap()
        );
        assert_eq!(
            size_mask_expected,
            registry.get_component_mask::<Size>().unwrap()
        );
        Ok(())
    }

    #[test]
    fn create_entities() -> Result<()> {
        let mut registry = Registry::default();
        registry.register_component::<f32>()?;
        let entity1 = registry.create_entity();
        let entity2 = registry.create_entity();

        assert_eq!(entity1.0, 0);
        assert_eq!(entity2.0, 1);
        assert_eq!(registry.entities_to_be_added.borrow().len(), 2);
        assert_eq!(
            registry.entities_to_be_added.borrow().contains(&entity1),
            true
        );
        assert_eq!(
            registry.entities_to_be_added.borrow().contains(&entity2),
            true
        );
        assert_eq!(registry.get_num_entities(), 2);

        // testing if components_vec is filled with default None values (nullptr)
        for (type_id, _) in registry.components.iter() {
            let components_vec = registry.components.get(type_id).unwrap();
            assert_eq!(components_vec.len(), 2);
            for el in components_vec.iter() {
                assert_eq!(el.is_none(), true);
            }
        }
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

        let type33 = registry.register_component::<Type33>();
        assert_eq!(type33.is_err(), true);
        Ok(())
    }
}
*/
