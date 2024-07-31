use crate::{
    framework::tileset::{TILESET_PATH_DIFFUSE, TILESET_PATH_NORMAL},
    game::objects::{cauldron, definitions::ColorDef},
};
use bevy::{asset::LoadState, color::palettes::tailwind, prelude::*};

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

    pub flag_meshes: [Handle<Mesh>; 2],
    pub flag_materials: [Handle<StandardMaterial>; 2],

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

pub fn load_object_assets(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cauldron_mesh = ass.load("objects.glb#Mesh2/Primitive0");
    let cauldron_fluid_mesh = ass.load("objects.glb#Mesh2/Primitive1");
    let cauldron_material = {
        materials.add(StandardMaterial {
            base_color: Color::hsl(0.0, 0.2, 0.1),
            ..Default::default()
        })
    };

    let cauldron_fluid_materials = ColorDef::VARIANTS.map(|color| {
        materials.add(StandardMaterial {
            base_color: cauldron::fluid_base_color(color).into(),
            ..Default::default()
        })
    });

    let camera_wall_mount = ass.load("objects.glb#Mesh0/Primitive0");
    let camera_rotating_mesh = ass.load("objects.glb#Mesh1/Primitive0");

    let camera_material = materials.add(StandardMaterial {
        // base_color_texture: Some(diffuse.clone()),
        // normal_map_texture: normal.clone(),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    let flag_meshes = [
        ass.load("objects.glb#Mesh3/Primitive0"),
        ass.load("objects.glb#Mesh3/Primitive1"),
    ];
    let flag_materials = [
        cauldron_material.clone(),
        materials.add(StandardMaterial {
            base_color: cauldron::fluid_base_color(ColorDef::Red).into(),
            ..Default::default()
        }),
    ];

    let dummy_cube_mesh = meshes.add(Cuboid::default());

    let map_base_texture = ass.load(TILESET_PATH_DIFFUSE);
    let map_norm_texture = ass.load(TILESET_PATH_NORMAL);

    let map_ground_material = materials.add(StandardMaterial {
        base_color_texture: Some(map_base_texture.clone()),
        normal_map_texture: Some(map_norm_texture.clone()),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    let map_wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(map_base_texture.clone()),
        normal_map_texture: Some(map_norm_texture.clone()),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    let dummy_cube_materials = ColorDef::VARIANTS.map(|col| {
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
    });

    cmd.insert_resource(GameObjectAssets {
        cauldron_mesh,
        cauldron_fluid_mesh,
        cauldron_material,
        cauldron_fluid_materials,
        camera_wall_mount,
        camera_rotating_mesh,
        camera_material,
        flag_meshes,
        flag_materials,
        map_base_texture,
        map_norm_texture,
        map_ground_material,
        map_wall_material,
        dummy_cube_mesh,
        dummy_cube_materials,
    });
}

#[rustfmt::skip]
pub fn are_object_assets_ready(
    ass: &AssetServer,
    assets: &GameObjectAssets,
) -> bool {
    ass.load_state(&assets.cauldron_mesh)        != LoadState::Loading &&
    ass.load_state(&assets.cauldron_fluid_mesh)  != LoadState::Loading &&
    ass.load_state(&assets.cauldron_material)    != LoadState::Loading &&
    ass.load_state(&assets.camera_wall_mount)    != LoadState::Loading &&
    ass.load_state(&assets.camera_rotating_mesh) != LoadState::Loading &&
    ass.load_state(&assets.camera_material)      != LoadState::Loading &&
    ass.load_state(&assets.map_base_texture)     != LoadState::Loading &&
    ass.load_state(&assets.map_norm_texture)     != LoadState::Loading &&
    ass.load_state(&assets.map_ground_material)  != LoadState::Loading &&
    ass.load_state(&assets.map_wall_material)    != LoadState::Loading &&
    ass.load_state(&assets.dummy_cube_mesh)      != LoadState::Loading &&
    assets.flag_meshes             .iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0 && 
    assets.flag_materials          .iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0 &&
    assets.cauldron_fluid_materials.iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0 && 
    assets.dummy_cube_materials    .iter().map(|u| (ass.load_state(u) == LoadState::Loading) as u32).sum::<u32>() == 0
}
