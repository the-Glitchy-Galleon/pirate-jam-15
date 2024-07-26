use bevy::{color::palettes::tailwind, prelude::*};

use crate::{
    framework::prelude::LevelAsset,
    game::objects::{
        camera::CameraObjBuilder,
        definitions::{ColorDef, ObjectDefKind},
    },
};

use super::objects::assets::GameObjectAssets;

#[derive(Component)]
pub struct InitLevel {
    handle: Handle<LevelAsset>,
}

pub fn load_preview_scene(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    already_init: Option<Res<UserDefinedStartupLevel>>,
) {
    if !already_init.is_some() {
        info!("Loading Preview Scene");
        cmd.spawn(InitLevel {
            handle: ass.load("level/preview.level"),
        });
    }
}

pub fn init_level(
    mut cmd: Commands,
    level_q: Query<(Entity, &InitLevel)>,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    levels: Res<Assets<LevelAsset>>,
    assets: Res<GameObjectAssets>,
) {
    use crate::framework::tileset::TILESET_PATH_DIFFUSE;
    use crate::framework::tileset::TILESET_PATH_NORMAL;

    for (ent, init) in level_q.iter() {
        if let Some(level) = levels.get(&init.handle) {
            info!("Initializing Level");
            cmd.entity(ent).despawn();

            let diffuse: Handle<Image> = ass.load(TILESET_PATH_DIFFUSE);
            let normal: Option<Handle<Image>> = TILESET_PATH_NORMAL.map(|f| ass.load(f));

            let handle: Handle<Mesh> = meshes.add(level.data().ground_mesh.clone());
            let collider = level.data().ground_collider.clone();

            let _ent = cmd
                .spawn((
                    PbrBundle {
                        mesh: handle,
                        material: mats.add(StandardMaterial {
                            base_color_texture: Some(diffuse.clone()),
                            normal_map_texture: normal.clone(),
                            perceptual_roughness: 0.9,
                            metallic: 0.0,
                            ..default()
                        }),
                        ..default()
                    },
                    collider,
                ))
                .id();

            let mut walls = vec![];
            for wall in &level.data().walls {
                let mesh = wall.mesh.clone();
                let collider = wall.collider.clone();
                let handle: Handle<Mesh> = meshes.add(mesh);
                walls.push(
                    cmd.spawn((
                        PbrBundle {
                            mesh: handle,
                            material: mats.add(StandardMaterial {
                                base_color_texture: Some(diffuse.clone()),
                                normal_map_texture: normal.clone(),
                                perceptual_roughness: 0.9,
                                metallic: 0.0,
                                ..default()
                            }),
                            transform: Transform::IDENTITY,
                            ..default()
                        },
                        collider,
                    ))
                    .id(),
                );
            }
            // as children because the map is scaled for now
            // cmd.entity(ent).push_children(&walls);

            for object in &level.data().objects {
                info!(
                    "Spawning {} {}",
                    object.color.as_str(),
                    object.kind.as_str()
                );
                match object.kind {
                    ObjectDefKind::Camera => {
                        let builder = CameraObjBuilder(object.clone());
                        builder.build(&mut cmd, &assets);
                    }
                    _ => {
                        cmd.spawn(PbrBundle {
                            mesh: meshes.add(Cuboid::default()),
                            material: mats.add(StandardMaterial {
                                base_color_texture: Some(diffuse.clone()),
                                normal_map_texture: normal.clone(),
                                perceptual_roughness: 0.9,
                                metallic: 0.0,
                                base_color: match object.color {
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
                            }),
                            transform: Transform::IDENTITY
                                .with_translation(object.position)
                                .with_rotation(Quat::from_rotation_y(object.rotation)),
                            ..default()
                        });
                    }
                }
            }

            info!("Done");

            cmd.insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 250.,
            });
            cmd.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 2_500.0,
                    shadows_enabled: true,

                    ..Default::default()
                },
                transform: Transform::IDENTITY
                    .with_translation(Vec3::new(5.0, 10.0, -5.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            });
        }
    }
}

#[derive(Resource)]
pub struct UserDefinedStartupLevel(pub String);

pub fn load_user_defined_startup_level(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    level: Res<UserDefinedStartupLevel>,
) {
    cmd.spawn(InitLevel {
        handle: ass.load(format!("level/{}.level", &level.0)),
    });
}
