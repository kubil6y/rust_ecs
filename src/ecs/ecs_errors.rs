use thiserror::Error;

#[derive(Debug, Error)]
pub enum EcsErrors {
    #[error("Max component count has been reached. Try removing a component or increase MAXCOMPONENTS")]
    MaxComponentReached,
}
