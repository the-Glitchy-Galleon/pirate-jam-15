use crate::{
    framework::{level_asset::LevelAsset, navmesh},
    game::{
        collision_groups::{ACTOR_GROUP, GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
        objects::{
            assets::GameObjectAssets,
            camera::CameraObjBuilder,
            definitions::{ColorDef, ObjectDefKind},
            destructible_target_test::DestructibleTargetTestBuilder,
            physics_cubes_test::PhysicsCubeTestBuilder,
        },
        LevelResources,
    },
};
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_rapier3d::prelude::*;
use vleue_navigator::NavMesh;

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
    init: Query<(Entity, &InitLevel)>,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    level: Res<Assets<LevelAsset>>,
    assets: Res<GameObjectAssets>,
    mut navs: ResMut<Assets<NavMesh>>,
) {
    use crate::framework::tileset::TILESET_PATH_DIFFUSE;
    use crate::framework::tileset::TILESET_PATH_NORMAL;

    for (ent, init) in init.iter() {
        if let Some(level) = level.get(&init.handle) {
            info!("Initializing Level");
            cmd.entity(ent).despawn();

            let diffuse: Handle<Image> = ass.load(TILESET_PATH_DIFFUSE);
            let normal: Option<Handle<Image>> = TILESET_PATH_NORMAL.map(|f| ass.load(f));

            let ground_mesh: Handle<Mesh> = meshes.add(level.data().baked_ground_mesh.clone());
            let collider = level.data().baked_ground_collider.clone();

            // Spawn ground Mesh

            let _ent = cmd
                .spawn((
                    PbrBundle {
                        mesh: ground_mesh,
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
                    CollisionGroups::new(GROUND_GROUP, ACTOR_GROUP | TARGET_GROUP),
                ))
                .id();

            let mut walls = vec![];
            for wall in &level.data().baked_walls {
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
                        CollisionGroups::new(WALL_GROUP, ACTOR_GROUP | TARGET_GROUP),
                    ))
                    .id(),
                );
            }

            // create navmesh
            let dims = level.data().tilemap.dims();

            let walls = level
                .data()
                .tilemap
                .faces()
                .map(|face| face.wall_height > 0)
                .collect::<Vec<_>>();

            let objects = vec![];

            let handle = navs.reserve_handle();

            let (vertices, polygons) =
                navmesh::create_grid_mesh_with_holes(dims, &walls, &objects, 0.4);

            let mut navmesh =
                NavMesh::from_polyanya_mesh(polyanya::Mesh::new(vertices, polygons).unwrap());

            let transform =
                Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

            navmesh.set_transform(transform);

            // use rand::Rng;
            // let dims_f32 = dims.as_vec2();
            // let offset_2 = dims_f32 * -0.5;
            // let offset = Vec3::new(offset_2.x, 0.0, offset_2.y);
            // let mut rng = rand::thread_rng();
            // for y in 0..dims.y {
            //     for x in 0..dims.x {
            //         let pos = Vec3::new(x as f32, rng.gen_range(0.0..10.0), y as f32)
            //             + offset
            //             + Vec3::splat(0.5);
            //         let point = navmesh.transform().transform_point(pos).xy();
            //         print!(
            //             "{:02.02},{:02.02} => {}",
            //             point.x,
            //             point.y,
            //             if navmesh.is_in_mesh(point) { "X" } else { "o" }
            //         );
            //     }
            //     println!();
            // }

            navs.insert(handle.id(), navmesh);
            cmd.insert_resource(LevelResources { navmesh: handle });

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
                    ObjectDefKind::DestructibleTargetTest => {
                        let builder = DestructibleTargetTestBuilder(object.clone());
                        builder.build(&mut cmd, &assets);
                    }
                    ObjectDefKind::PhysicsCubesTest => {
                        let builder = PhysicsCubeTestBuilder(object.clone());
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
