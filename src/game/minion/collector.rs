use bevy::{prelude::*, utils::HashMap};

use super::{MinionKind, MinionState};

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

#[derive(Clone, Debug, Component, Default)]
pub struct MinionInteractionRequirement {
    pub counts: HashMap<MinionKind, u32>,
    pub is_satisfied: bool,
}

impl MinionInteractionRequirement {
    pub fn new(counts: HashMap<MinionKind, u32>) -> Self {
        Self {
            counts,
            is_satisfied: false,
        }
    }
}

pub fn update_minion_interaction_requirements(
    mut buffer: Local<HashMap<MinionKind, u32>>,
    mut requirements_q: Query<(Entity, &mut MinionInteractionRequirement)>,
    minion_q: Query<(&MinionKind, &MinionState)>,
) {
    for (ent, mut req) in requirements_q.iter_mut() {
        buffer.clear();

        minion_q
            .iter()
            .filter(|(_, st)| **st == MinionState::Interracting(ent))
            .map(|(kind, _)| *kind)
            .for_each(|kind| *buffer.entry(kind).or_default() += 1);

        req.is_satisfied = req
            .counts
            .iter()
            .filter(|(_, cnt)| **cnt > 0)
            .all(|(k, cnt)| buffer.get(k).map(|x| *x).unwrap_or_default() >= *cnt);
    }
}
