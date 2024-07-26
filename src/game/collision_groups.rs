#![cfg_attr(rustfmt, rustfmt_skip)]
use bevy_rapier3d::prelude::*;

pub const G_ALL:     Group = Group::all();

#[allow(unused)]
pub const G_NONE:    Group = Group::NONE;

pub const G_SENSOR:  Group = Group::GROUP_1;

pub const G_PLAYER:  Group = Group::GROUP_2;
pub const G_MINION:  Group = Group::GROUP_3;
pub const G_OBJECT:  Group = Group::GROUP_4;
pub const G_WALL:    Group = Group::GROUP_5;
pub const G_GROUND:  Group = Group::GROUP_6;
