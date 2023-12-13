use thiserror::Error;

#[derive(Debug, Error)]
pub enum EcsErrors {
    #[error(
        "Max component count has been reached. Try removing a component or increase MAXCOMPONENTS"
    )]
    MaxComponentReached,
    #[error("Component is not registered")]
    ComponentDoesNotExist,
    #[error("Component mask does not exist")]
    ComponentMaskDoesNotExist,
    #[error("Entity component mask does not exist")]
    EntityComponentMaskDoesNotExist,
    #[error("System is not registered")]
    SystemDoesNotExist,
}
