use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

macro_rules! cool_enum { // pfft, strum..
    (
        $(#[$meta:meta])*
        $v:vis enum $n:ident ($t:ty, $c:literal) { $(
            $vn:ident: $vs:literal
        ),* $(,)? }
    ) => {
        $(#[$meta])*
        $v enum $n {$(
            $vn
        ),* }

        impl $n {
            pub const COUNT: usize = $c;
            pub const VARIANTS: [Self; Self::COUNT] = [ $( $n :: $vn),*];
            pub const NAMES: [&'static str; Self::COUNT] = [$($vs),*];
            pub fn as_str(self) -> &'static str {
                Self::NAMES[self as $t as usize]
            }
        }
    };
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct ObjectDef {
    pub kind: ObjectDefKind,
    pub position: Vec3,
    pub rotation: f32,
    pub color: ColorDef,
    pub obj_refs: Vec<u32>,
    pub pos_refs: Vec<Vec3>,
    pub tags: Vec<Tag>,
}

cool_enum! {

    #[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize, PartialEq, Eq)]
    #[repr(u32)]
    #[rustfmt::skip]
    pub enum ObjectDefKind (u32, 3) {
        Cauldron         : "Tinting Cauldron",
        Camera           : "Camera",
        LaserGrid        : "Laser Grid",
        // ControlPanel     : "Control Panel",
        // PressurePlate    : "Pressure Plate",
        // Key              : "Key",
        // KeyDoor          : "Locked Door",
        // Barrier          : "Locked Barrier",
        // PowerOutlet      : "Power Outlet",
        // EmptySocket      : "Empty Socket",
        // ExplosiveBarrel  : "Explosive Barrel",
        // Anglerfish       : "Anglerfish",
        // Ventilation      : "Ventilation",
        // Well             : "Well",
        // BigHole          : "Big Hole",
        // LaserDrill       : "Laser Drill",
        // VaultDoor        : "Vault Door",
        // Painting         : "Pretentious Painting",
        // Vase             : "Antique Vase",
        // Ingot            : "Gold Ingot",
        // Sculpture        : "Sculpture",
        // Book             : "Book",
        // Relic            : "Relic",
        // WineBottle       : "Wine Bottle",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
#[repr(u8)]
#[rustfmt::skip]
pub enum ColorDef {
    Void    = 0b000,
    Red     = 0b001,
    Green   = 0b010,
    Blue    = 0b100,
    Yellow  = 0b011,
    Magenta = 0b101,
    Cyan    = 0b110,
    White   = 0b111,
}

impl ColorDef {
    pub fn contains(&self, color: ColorDef) -> bool {
        (*self as u8 & color as u8) == color as u8
    }
    #[rustfmt::skip]
    pub fn as_str(self) -> &'static str {
        match self {
            ColorDef::Void    => "Void",
            ColorDef::Red     => "Red",
            ColorDef::Green   => "Green",
            ColorDef::Blue    => "Blue",
            ColorDef::Yellow  => "Yellow",
            ColorDef::Magenta => "Magenta",
            ColorDef::Cyan    => "Cyan",
            ColorDef::White   => "White",
        }
    }
}

impl Add<ColorDef> for ColorDef {
    type Output = ColorDef;
    fn add(self, rhs: ColorDef) -> Self::Output {
        unsafe { core::mem::transmute(self as u8 | rhs as u8) }
    }
}

impl Sub<ColorDef> for ColorDef {
    type Output = ColorDef;
    fn sub(self, rhs: ColorDef) -> Self::Output {
        unsafe { core::mem::transmute(self as u8 ^ rhs as u8) }
    }
}

// #[derive(Clone, Copy, PartialEq, Eq)]
// #[repr(u8)]
// pub enum Ability {
//     Strong,
//     Agile,
//     Stealthy,
//     Conductive,
//     Heavy,
//     Fleeting,
// }

// pub struct Minion {
//     color: Color,
// }

// impl Minion {
//     #[rustfmt::skip]
//     pub fn has_ability(&self, ability: Ability) -> bool {
//         let col = self.color;
//         match ability {
//             Ability::Strong     => col == Color::White || col == Color::Red    || col == Color::Magenta,
//             Ability::Agile      => col == Color::White || col == Color::Green  || col == Color::Yellow,
//             Ability::Stealthy   => col == Color::White || col == Color::Blue   || col == Color::Cyan,
//             Ability::Conductive => col == Color::White || col == Color::Yellow,
//             Ability::Heavy      => col == Color::White || col == Color::Magenta,
//             Ability::Fleeting   => col == Color::White || col == Color::Cyan,
//         }
//     }
// }

#[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tag {}
