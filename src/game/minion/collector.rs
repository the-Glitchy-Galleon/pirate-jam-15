use bevy::{prelude::*, utils::HashMap};

use super::MinionKind;

#[derive(Component, Reflect, Debug)]
pub struct MinionStorage {
    storage: HashMap<MinionKind, u32>,
}

impl MinionStorage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn add_minion(&mut self, ty: MinionKind) {
        *self.storage.entry(ty).or_default() += 1;
    }

    pub fn extract_minion(&mut self, ty: MinionKind) -> bool {
        let cnt = self.storage.entry(ty).or_default();

        if *cnt == 0 {
            return false;
        }

        *cnt -= 1;

        true
    }
}