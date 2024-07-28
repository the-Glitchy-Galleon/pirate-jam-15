use crate::{
    framework::{
        level_asset::LevelAsset,
        loading_queue::{AssetLoadingCompleted, WatchAssetLoading},
        navmesh,
    },
    game::{
        collision_groups::{ACTOR_GROUP, GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
        objects::{self, assets::GameObjectAssets, definitions::ObjectDefKind},
        player::AddPlayerRespawnEvent,
        LevelResources,
    },
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use vleue_navigator::NavMesh;

pub fn load_preview_scene(
    ass: Res<AssetServer>,
    already_init: Option<Res<UserDefinedStartupLevel>>,
    mut load: EventWriter<WatchAssetLoading<LevelAsset>>,
) {
    if !already_init.is_some() {
        info!("Loading Preview Scene");
        load.send(WatchAssetLoading::new(ass.load("level/preview.level")));
    }
}

pub fn init_level(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut navs: ResMut<Assets<NavMesh>>,
    mut load: EventReader<AssetLoadingCompleted<LevelAsset>>,
    level: Res<Assets<LevelAsset>>,
    assets: Res<GameObjectAssets>,
    mut respawn: EventWriter<AddPlayerRespawnEvent>,
) {
    let Some(load) = load.read().last() else {
        return;
    };
    let Some(level) = level.get(&load.handle) else {
        error!("Failed to get LevelAsset data");
        return;
    };

    let ground_mesh: Handle<Mesh> = meshes.add(level.data().baked_ground_mesh.clone());
    let collider = level.data().baked_ground_collider.clone();

    // Spawn ground Mesh
    cmd.spawn((
        PbrBundle {
            mesh: ground_mesh,
            material: assets.map_ground_material.clone(),
            ..default()
        },
        collider,
        CollisionGroups::new(GROUND_GROUP, ACTOR_GROUP | TARGET_GROUP),
    ));

    let mut walls = vec![];
    for wall in &level.data().baked_walls {
        let mesh = wall.mesh.clone();
        let collider = wall.collider.clone();
        let handle: Handle<Mesh> = meshes.add(mesh);
        walls.push(
            cmd.spawn((
                PbrBundle {
                    mesh: handle,
                    material: assets.map_wall_material.clone(),
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

    let (vertices, polygons) = navmesh::create_grid_mesh_with_holes(dims, &walls, &objects, 0.4);

    let mut navmesh = NavMesh::from_polyanya_mesh(polyanya::Mesh::new(vertices, polygons).unwrap());

    let transform = Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

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

    let spawnpoints = level
        .data()
        .objects
        .iter()
        .filter(|o| o.kind == ObjectDefKind::SpawnPoint)
        .map(|o| (o.position, o.number, o.number == 0))
        .collect::<Vec<_>>();

    let lowest_respawn_pos = spawnpoints
        .iter()
        .filter(|o| o.2)
        .min_by(|a, b| a.1.cmp(&b.1))
        .map(|o| o.0)
        .unwrap_or(Vec3::ZERO + Vec3::Y * 7.0);

    cmd.insert_resource(LevelResources {
        navmesh: Some(handle),
        spawnpoints: Some(spawnpoints),
    });

    for object in &level.data().objects {
        let _ent = objects::spawn_object(&mut cmd, object, assets.as_ref());
    }

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

    info!("Spawning player at {lowest_respawn_pos:?}");
    respawn.send(AddPlayerRespawnEvent {
        position: lowest_respawn_pos,
    });
}

#[derive(Resource)]
pub struct UserDefinedStartupLevel(pub String);

pub fn load_user_defined_startup_level(
    ass: Res<AssetServer>,
    level: Res<UserDefinedStartupLevel>,
    mut load: EventWriter<WatchAssetLoading<LevelAsset>>,
) {
    load.send(WatchAssetLoading::new(
        ass.load(format!("level/{}.level", &level.0)),
    ));
}
