use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

macro_rules! object_enum { // pfft, strum..
    (
        $(#[$meta:meta])*
        $v:vis enum $n:ident ($t:ty, $c:literal) { $(
            $vn:ident = $vi:literal : $vs:literal
        ),* $(,)? }
    ) => {

        $(#[$meta])*
        #[repr($t)]
        $v enum $n {$(
            $vn = $vi
        ),* }

        impl $n {
            pub const COUNT: usize = $c;
            pub const VARIANTS: [Self; Self::COUNT] = [ $( $n :: $vn),*];

            pub fn as_str(&self) -> &'static str {
                match self {$(
                    $n :: $vn => $vs
                ),*}
            }
        }
        impl AsRef<str> for $n {
            fn as_ref(&self) -> &str {
                self.as_str()
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

object_enum! {
    #[derive(Debug, Clone, Copy, Reflect, Serialize, Deserialize, PartialEq, Eq)]
    #[rustfmt::skip]
    pub enum ObjectDefKind (u32, 6) {
        SpawnPoint         = 0x0001 : "Spawn Point",
        Cauldron           = 0x0101 : "Tinting Cauldron",
        Camera             = 0x0102 : "Camera",
        LaserGrid          = 0x0103 : "Laser Grid",
        // ControlPanel       = 0x0104 : "Control Panel",
        // PressurePlate      = 0x0105 : "Pressure Plate",
        // Key                = 0x0106 : "Key",
        // KeyDoor            = 0x0107 : "Locked Door",
        // Barrier            = 0x0108 : "Locked Barrier",
        // PowerOutlet        = 0x0109 : "Power Outlet",
        // EmptySocket        = 0x010A : "Empty Socket",
        // ExplosiveBarrel    = 0x010B : "Explosive Barrel",
        // Anglerfish         = 0x010C : "Anglerfish",
        // Ventilation        = 0x010D : "Ventilation",
        // Well               = 0x0201  : "Well",
        // BigHole            = 0x0202  : "Big Hole",
        // LaserDrill         = 0x0203  : "Laser Drill",
        // VaultDoor          = 0x0204  : "Vault Door",
        // Painting           = 0x0205  : "Pretentious Painting",
        // Vase               = 0x0206  : "Antique Vase",
        // Ingot              = 0x0207  : "Gold Ingot",
        // Sculpture          = 0x0208  : "Sculpture",
        // Book               = 0x0209  : "Book",
        // Relic              = 0x020A  : "Relic",
        // WineBottle         = 0x020B  : "Wine Bottle",
        DestructibleTargetTest = 0xFF01 : "Destructible Target Test",
        PhysicsCubesTest       = 0xFF02 : "Physics Cubes Test",
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
