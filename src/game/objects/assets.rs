use crate::{
    framework::tileset::{TILESET_PATH_DIFFUSE, TILESET_PATH_NORMAL},
    game::objects::definitions::ColorDef,
};
use bevy::{color::palettes::tailwind, prelude::*};

#[derive(Resource)]
pub struct GameObjectAssets {
    pub camera_wall_mount: Handle<Mesh>,
    pub camera_rotating_mesh: Handle<Mesh>,
    pub camera_material: Handle<StandardMaterial>,

    pub map_base_texture: Handle<Image>,
    pub map_norm_texture: Handle<Image>,
    pub map_ground_material: Handle<StandardMaterial>,
    pub map_wall_material: Handle<StandardMaterial>,

    pub dummy_cube_mesh: Handle<Mesh>,
    pub dummy_cube_materials: [Handle<StandardMaterial>; ColorDef::COUNT],
}

impl FromWorld for GameObjectAssets {
    fn from_world(world: &mut World) -> Self {
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
