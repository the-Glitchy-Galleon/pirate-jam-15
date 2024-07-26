use crate::{
    framework::level_asset::LevelAsset,
    game::{
        collision_groups::{ACTOR_GROUP, GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
        minion::{
            collector::MinionInteractionRequirement, destructible_target::DestructibleTargetBundle,
            MinionKind,
        },
        objects::{
            assets::GameObjectAssets,
            camera::CameraObjBuilder,
            definitions::{ColorDef, ObjectDefKind},
        },
        LevelResources,
    },
};
use bevy::{color::palettes::tailwind, prelude::*, utils::HashMap};
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

            let ground_mesh: Handle<Mesh> = meshes.add(level.data().ground_mesh.clone());
            let collider = level.data().ground_collider.clone();

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
                        CollisionGroups::new(WALL_GROUP, ACTOR_GROUP | TARGET_GROUP),
                        ActiveCollisionTypes::all(),
                    ))
                    .id(),
                );
            }
            // as children because the map is scaled for now
            // cmd.entity(ent).push_children(&walls);

            let handle = navs.reserve_handle();
            // let flattened_ground_mesh: Mesh = level.data().ground_mesh.flattened().into();
            let tilemap_dims = UVec2::new(128, 128);
            let mut navmesh = helpers::create_navmesh(tilemap_dims.x, tilemap_dims.y);

            let transform =
                Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    // .with_translation(Vec3::new(
                    //     tilemap_dims.x as f32 * -0.5,
                    //     tilemap_dims.y as f32 * -0.5,
                    //     0.0,
                    // ));
                    ;
            navmesh.set_transform(transform);

            navs.insert(handle.id(), navmesh);
            cmd.insert_resource(LevelResources { navmesh: handle });

            cmd.spawn((
                DestructibleTargetBundle {
                    requirement: {
                        let mut map = HashMap::new();
                        map.insert(MinionKind::Void, 2);

                        MinionInteractionRequirement::new(map)
                    },
                    ..default()
                },
                TransformBundle::from(Transform::from_xyz(4.0, 0.0, 4.0)),
                Collider::cuboid(1.0, 1.0, 1.0),
                CollisionGroups::new(TARGET_GROUP, GROUND_GROUP | ACTOR_GROUP),
            ));

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

mod helpers {
    use bevy::{prelude::*, utils::HashMap};
    use polyanya::{Polygon, Vertex};
    use vleue_navigator::NavMesh;

    pub fn create_navmesh(width: u32, height: u32) -> NavMesh {
        let mesh = create_polyanya_mesh(width, height);
        NavMesh::from_polyanya_mesh(polyanya::Mesh::new(mesh.0, mesh.1).unwrap())
    }

    pub fn create_polyanya_mesh(width: u32, height: u32) -> (Vec<Vertex>, Vec<Polygon>) {
        let mut vertices = Vec::new();
        let mut vertex_indices = HashMap::new();
        let mut polygons = Vec::new();

        let offset = Vec2::new(width as f32 * -0.5, height as f32 * -0.5);
        // Create vertices
        for y in 0..=height {
            for x in 0..=width {
                let index = vertices.len() as u32;
                let coords = Vec2::new(x as f32, y as f32);
                vertex_indices.insert((x, y), index);
                vertices.push(Vertex::new(coords + offset, vec![]));
            }
        }

        // Create polygons
        for y in 0..height {
            for x in 0..width {
                let v0 = *vertex_indices.get(&(x, y)).unwrap();
                let v1 = *vertex_indices.get(&(x + 1, y)).unwrap();
                let v2 = *vertex_indices.get(&(x + 1, y + 1)).unwrap();
                let v3 = *vertex_indices.get(&(x, y + 1)).unwrap();
                let polygon_index = polygons.len() as isize;

                let polygon = Polygon {
                    vertices: vec![v0, v1, v2, v3],
                    is_one_way: false,
                };
                polygons.push(polygon);

                vertices[v0 as usize].polygons.push(polygon_index);
                vertices[v1 as usize].polygons.push(polygon_index);
                vertices[v2 as usize].polygons.push(polygon_index);
                vertices[v3 as usize].polygons.push(polygon_index);
            }
        }

        (vertices, polygons)
    }
}
