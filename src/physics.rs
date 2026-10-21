use avian2d::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub(crate) enum GameLayer {
    #[default]
    Default,
    Ground,
}
