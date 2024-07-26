#![cfg_attr(rustfmt, rustfmt_skip)]
use bevy_rapier3d::prelude::*;

pub const DETECTION_GROUP: Group = Group::GROUP_1;
pub const ACTOR_GROUP: Group = Group::GROUP_2;
pub const GROUND_GROUP: Group = Group::GROUP_3;
pub const TARGET_GROUP: Group = Group::GROUP_4;

pub const G_ALL:     Group = Group::all();

pub const G_PLAYER:  Group = Group::GROUP_2;
pub const G_MINION:  Group = Group::GROUP_3;
pub const G_OBJECT:  Group = Group::GROUP_4;
pub const G_WALL:    Group = Group::GROUP_5;
pub const G_GROUND:  Group = Group::GROUP_6;
