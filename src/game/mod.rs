use crate::framework::prelude::{AudioPlugin, LevelAsset, LevelAssetLoader};
use bevy::{
    color::palettes::tailwind,
    prelude::*,
};
use bevy::{input::InputSystem, utils::HashMap};
use bevy_rapier3d::prelude::*;
use object_def::ColorDef;

mod kinematic_char;
mod minion;
pub mod object_def;
mod player;

pub use kinematic_char::*;
pub use minion::*;
pub use player::*;
use vleue_navigator::{NavMesh, VleueNavigatorPlugin};

#[derive(Debug)]
#[derive(Resource)]
pub struct LevelResources {
    pub navmesh: Handle<NavMesh>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            AudioPlugin,
            VleueNavigatorPlugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        ));

        app.register_type::<CharacterWalkControl>()
            .register_type::<PlayerCollector>()
            .register_type::<CharacterWalkState>()
            .register_type::<MinionKind>()
            .register_type::<MinionStorage>()
            .register_type::<MinionState>()
            .register_type::<MinionTarget>()
            .register_type::<MinionThrowTarget>()
            .register_type::<MinionInteractionRequirement>();

        app.insert_resource(MinionStorageInput {
            chosen_ty: MinionKind::Void,
            want_to_throw: false,
            to_where: MinionThrowTarget::Location(Vec3::ZERO),
            do_pickup: false,
        });

        // app.insert_resource(LevelResources {
        //     navmesh: Handle::inva
        // });

        /* Setup */
        app.add_systems(Startup, spawn_gameplay_camera)
            .add_systems(Startup, setup_physics)
            .add_systems(Startup, setup_player);

        /* Common systems */
        app.add_systems(FixedUpdate, update_kinematic_character);

        /* Minion systems */
        app.add_systems(Update, cleanup_minion_state)
            .add_systems(Update, update_minion_state)
            .add_systems(Update,
                minion_update_path
                .run_if(resource_exists::<LevelResources>)
                .after(update_minion_state)
            )
            .add_systems(PostUpdate,
                minion_build_path
                .run_if(resource_exists::<LevelResources>)
                .after(TransformSystem::TransformPropagate)
            )
            .add_systems(Update,
                minion_walk
                .run_if(resource_exists::<LevelResources>)
            )
            .add_systems(Update, walk_target_update.after(update_minion_state))
            .add_systems(
                Update,
                update_minion_interaction_requirements.after(update_minion_state),
            )
            .add_systems(Update, update_destructble_target)
            .add_systems(Update, debug_navmesh);

        /* Player systems */
        app.add_systems(PreUpdate, player_controls.after(InputSystem))
            .add_systems(Update, minion_storage_throw)
            .add_systems(Update, minion_storage_pickup);

        app.init_asset::<LevelAsset>()
            .init_asset_loader::<LevelAssetLoader>();

        app.add_systems(Startup, load_preview_scene);
        // app.add_systems(Update, init_level);
    }
}

pub fn spawn_gameplay_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-30.0, 30.0, 30.0)
            .looking_at(Vec3::new(10.0, 0.0, 7.0), Vec3::Y),
        ..Default::default()
    });
}

fn setup_physics(
    mut navs: ResMut<Assets<NavMesh>>,
    mut commands: Commands
) {
    /*
     * Ground
     */
    let ground_size = 200.1;
    let ground_height = 0.1;

    let mesh = polyanya::Mesh::new(
        vec![
            polyanya::Vertex::new(Vec2::new(0., 6.), vec![0, -1]),           // 0
            polyanya::Vertex::new(Vec2::new(2., 5.), vec![0, -1, 2]),        // 1
            polyanya::Vertex::new(Vec2::new(5., 7.), vec![0, 2, -1]),        // 2
            polyanya::Vertex::new(Vec2::new(5., 8.), vec![0, -1]),           // 3
            polyanya::Vertex::new(Vec2::new(0., 8.), vec![0, -1]),           // 4
            polyanya::Vertex::new(Vec2::new(1., 4.), vec![1, -1]),           // 5
            polyanya::Vertex::new(Vec2::new(2., 1.), vec![1, -1]),           // 6
            polyanya::Vertex::new(Vec2::new(4., 1.), vec![1, -1]),           // 7
            polyanya::Vertex::new(Vec2::new(4., 2.), vec![1, -1, 2]),        // 8
            polyanya::Vertex::new(Vec2::new(2., 4.), vec![1, 2, -1]),        // 9
            polyanya::Vertex::new(Vec2::new(7., 4.), vec![2, -1, 4]),        // 10
            polyanya::Vertex::new(Vec2::new(10., 7.), vec![2, 4, 6, -1, 3]), // 11
            polyanya::Vertex::new(Vec2::new(7., 7.), vec![2, 3, -1]),        // 12
            polyanya::Vertex::new(Vec2::new(11., 8.), vec![3, -1]),          // 13
            polyanya::Vertex::new(Vec2::new(7., 8.), vec![3, -1]),           // 14
            polyanya::Vertex::new(Vec2::new(7., 0.), vec![5, 4, -1]),        // 15
            polyanya::Vertex::new(Vec2::new(11., 3.), vec![4, 5, -1]),       // 16
            polyanya::Vertex::new(Vec2::new(11., 5.), vec![4, -1, 6]),       // 17
            polyanya::Vertex::new(Vec2::new(12., 0.), vec![5, -1]),          // 18
            polyanya::Vertex::new(Vec2::new(12., 3.), vec![5, -1]),          // 19
            polyanya::Vertex::new(Vec2::new(13., 5.), vec![6, -1]),          // 20
            polyanya::Vertex::new(Vec2::new(13., 7.), vec![6, -1]),          // 21
            polyanya::Vertex::new(Vec2::new(1., 3.), vec![1, -1]),           // 22
        ],
        vec![
            polyanya::Polygon::new(vec![0, 1, 2, 3, 4], true),           // 0
            polyanya::Polygon::new(vec![5, 22, 6, 7, 8, 9], true),       // 1
            polyanya::Polygon::new(vec![1, 9, 8, 10, 11, 12, 2], false), // 2
            polyanya::Polygon::new(vec![12, 11, 13, 14], true),          // 3
            polyanya::Polygon::new(vec![10, 15, 16, 17, 11], false),     // 4
            polyanya::Polygon::new(vec![15, 18, 19, 16], true),          // 5
            polyanya::Polygon::new(vec![11, 17, 20, 21], true),          // 6
        ],
    ).unwrap();

    let handle = navs.reserve_handle();
    let mut navmesh = NavMesh::from_polyanya_mesh(mesh);
    navmesh.set_transform(Transform::from_rotation(Quat::from_rotation_x(
        -std::f32::consts::FRAC_PI_2
    )));

    navs.insert(handle.id(), navmesh);
    commands.insert_resource(LevelResources {
        navmesh: handle,
    });

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -ground_height, 0.0)),
        Collider::cuboid(ground_size, ground_height, ground_size),
    ));

    commands.spawn((
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
        Sensor,
    ));

    /*
     * Create the cubes
     */
    let num = 2;
    let rad = 1.0;

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    let centerz = shift * (num / 2) as f32;

    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
    let mut color = 0;
    let colors = [
        Hsla::hsl(220.0, 1.0, 0.3),
        Hsla::hsl(180.0, 1.0, 0.3),
        Hsla::hsl(260.0, 1.0, 0.7),
    ];

    for j in 0usize..2 {
        for i in 0..num {
            for k in 0usize..num {
                let x = i as f32 * shift - centerx + offset;
                let y = j as f32 * shift + centery + 3.0;
                let z = k as f32 * shift - centerz + offset;
                color += 1;

                commands.spawn((
                    TransformBundle::from(Transform::from_xyz(x, y, z)),
                    RigidBody::Dynamic,
                    Collider::cuboid(rad, rad, rad),
                    ColliderDebugColor(colors[color % 3]),
                ));
            }
        }

        offset -= 0.05 * rad * (num as f32 - 1.0);
    }
}

#[derive(Component)]
struct InitLevel {
    handle: Handle<LevelAsset>,
}

fn load_preview_scene(mut cmd: Commands, ass: Res<AssetServer>) {
    cmd.spawn(InitLevel {
        handle: ass.load("level/preview.level"),
    });
}

fn init_level(
    mut cmd: Commands,
    level_q: Query<(Entity, &InitLevel)>,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    levels: Res<Assets<LevelAsset>>,
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
                    transform: Transform::IDENTITY.with_scale(Vec3::ONE * 2.0),
                    ..default()
                },
                collider,
            ))
            .with_children(|parent| {
                // as children because the map is scaled for now

                for wall in &level.data().walls {
                    let mesh = wall.mesh.clone();
                    let collider = wall.collider.clone();
                    let handle: Handle<Mesh> = meshes.add(mesh);
                    parent.spawn((
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
                    ));
                }
                for object in &level.data().objects {
                    info!(
                        "Spawning {} {}",
                        object.color.as_str(),
                        object.kind.as_str()
                    );
                    parent.spawn(PbrBundle {
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
                            .with_scale(Vec3::splat(2.0))
                            .with_rotation(Quat::from_rotation_y(object.rotation)),
                        ..default()
                    });
                }
            });

            info!("Done");
        }
    }
}
