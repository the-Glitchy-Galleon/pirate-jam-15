use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component, Reflect)]
pub enum MinionKind {
    Spoink,
    Doink,
    Woink,
}