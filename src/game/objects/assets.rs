use crate::{
    framework::tileset::{TILESET_PATH_DIFFUSE, TILESET_PATH_NORMAL},
    game::objects::{cauldron, definitions::ColorDef},
};
use bevy::{color::palettes::tailwind, prelude::*};

macro_rules! load {
    ($world:expr, $str:literal) => {{
        let ass = $world.resource::<AssetServer>();
        ass.load($str)
    }};
}

#[derive(Resource)]
pub struct GameObjectAssets {
    pub cauldron_mesh: Handle<Mesh>,
    pub cauldron_fluid_mesh: Handle<Mesh>,
    pub cauldron_material: Handle<StandardMaterial>,
    cauldron_fluid_materials: [Handle<StandardMaterial>; ColorDef::COUNT],
    pub camera_wall_mount: Handle<Mesh>,
    pub camera_rotating_mesh: Handle<Mesh>,
    pub camera_material: Handle<StandardMaterial>,

    pub map_base_texture: Handle<Image>,
    pub map_norm_texture: Handle<Image>,
    pub map_ground_material: Handle<StandardMaterial>,
    pub map_wall_material: Handle<StandardMaterial>,

    pub dummy_cube_mesh: Handle<Mesh>,
    dummy_cube_materials: [Handle<StandardMaterial>; ColorDef::COUNT],
}

impl GameObjectAssets {
    #[rustfmt::skip]
    pub fn cauldron_fluid_material(&self, color: ColorDef) -> Handle<StandardMaterial> {
        match color {
            ColorDef::Void    => self.cauldron_fluid_materials[0].clone(),
            ColorDef::Red     => self.cauldron_fluid_materials[1].clone(),
            ColorDef::Green   => self.cauldron_fluid_materials[2].clone(),
            ColorDef::Blue    => self.cauldron_fluid_materials[3].clone(),
            ColorDef::Yellow  => self.cauldron_fluid_materials[4].clone(),
            ColorDef::Magenta => self.cauldron_fluid_materials[5].clone(),
            ColorDef::Cyan    => self.cauldron_fluid_materials[6].clone(),
            ColorDef::White   => self.cauldron_fluid_materials[7].clone(),
        }
    }
    #[rustfmt::skip]
    pub fn dummy_cube_material(&self, color: ColorDef) -> Handle<StandardMaterial> {
        match color {
            ColorDef::Void    => self.dummy_cube_materials[0].clone(),
            ColorDef::Red     => self.dummy_cube_materials[1].clone(),
            ColorDef::Green   => self.dummy_cube_materials[2].clone(),
            ColorDef::Blue    => self.dummy_cube_materials[3].clone(),
            ColorDef::Yellow  => self.dummy_cube_materials[4].clone(),
            ColorDef::Magenta => self.dummy_cube_materials[5].clone(),
            ColorDef::Cyan    => self.dummy_cube_materials[6].clone(),
            ColorDef::White   => self.dummy_cube_materials[7].clone(),
        }
    }
}

impl FromWorld for GameObjectAssets {
    fn from_world(world: &mut World) -> Self {
        let cauldron_mesh = load!(world, "objects.glb#Mesh2/Primitive0");
        let cauldron_fluid_mesh = load!(world, "objects.glb#Mesh2/Primitive1");
        let cauldron_material = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            materials.add(StandardMaterial {
                base_color: Color::hsl(0.0, 0.2, 0.1),
                ..Default::default()
            })
        };

        let cauldron_fluid_materials = ColorDef::VARIANTS.map(|color| {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            materials.add(StandardMaterial {
                base_color: cauldron::fluid_base_color(color).into(),
                ..Default::default()
            })
        });

        let camera_wall_mount = {
            let ass = world.resource::<AssetServer>();
            ass.load("objects.glb#Mesh0/Primitive0")
        };
        let camera_rotating_mesh = {
            let ass = world.resource::<AssetServer>();
            ass.load("objects.glb#Mesh1/Primitive0")
        };

        let camera_material = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            materials.add(StandardMaterial {
                // base_color_texture: Some(diffuse.clone()),
                // normal_map_texture: normal.clone(),
                perceptual_roughness: 0.9,
                metallic: 0.0,
                ..default()
            })
        };

        let dummy_cube_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Cuboid::default())
        };

        let map_base_texture = world.resource::<AssetServer>().load(TILESET_PATH_DIFFUSE);
        let map_norm_texture = world.resource::<AssetServer>().load(TILESET_PATH_NORMAL);

        let map_ground_material =
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color_texture: Some(map_base_texture.clone()),
                    normal_map_texture: Some(map_norm_texture.clone()),
                    perceptual_roughness: 0.9,
                    metallic: 0.0,
                    ..default()
                });

        let map_wall_material =
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color_texture: Some(map_base_texture.clone()),
                    normal_map_texture: Some(map_norm_texture.clone()),
                    perceptual_roughness: 0.9,
                    metallic: 0.0,
                    ..default()
                });

        let dummy_cube_materials = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            ColorDef::VARIANTS.map(|col| {
                materials.add(StandardMaterial {
                    base_color_texture: Some(map_base_texture.clone()),
                    normal_map_texture: Some(map_norm_texture.clone()),
                    perceptual_roughness: 0.9,
                    metallic: 0.0,
                    base_color: match col {
                        ColorDef::Void => tailwind::GRAY_500,
                        ColorDef::Red => tailwind::RED_500,
                        ColorDef::Green => tailwind::GREEN_500,
                        ColorDef::Blue => tailwind::BLUE_500,
                        ColorDef::Yellow => tailwind::YELLOW_500,
                        ColorDef::Magenta => tailwind::PURPLE_500,
                        ColorDef::Cyan => tailwind::CYAN_500,
                        ColorDef::White => tailwind::GREEN_100,
                    }
                    .into(),
                    ..default()
                })
            })
        };

        Self {
            cauldron_mesh,
            cauldron_fluid_mesh,
            cauldron_material,
            cauldron_fluid_materials,
            camera_wall_mount,
            camera_rotating_mesh,
            camera_material,
            map_base_texture,
            map_norm_texture,
            map_ground_material,
            map_wall_material,
            dummy_cube_mesh,
            dummy_cube_materials,
        }
    }
}
