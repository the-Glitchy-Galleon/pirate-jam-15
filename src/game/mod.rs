use bevy::{
    color::palettes::tailwind,
    input::InputSystem,
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
use bevy_rapier3d::prelude::*;
use object_def::ColorDef;
pub use player_minion::*;
pub use player_movement::*;
use crate::framework::prelude::{AudioPlugin, LevelAsset, LevelAssetLoader};

pub mod object_def;
pub mod player_minion;
pub mod player_movement;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerDirection>()
            .register_type::<PlayerCollector>()
            .register_type::<MovementInput>()
            .register_type::<MinionKind>()
            .register_type::<MinionStorage>();

        app.insert_resource(PlayerDirection(Dir3::X))
            .insert_resource(MinionInput {
                chosen_ty: MinionKind::Doink,
                want_to_throw: false,
                to_where: Vec3::ZERO,
            });

        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .init_resource::<MovementInput>()
        .add_systems(Startup, spawn_gameplay_camera)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, setup_player)
        .add_systems(PreUpdate, mouse_tap.after(InputSystem))
        .add_systems(FixedUpdate, player_movement)
        .add_systems(Update, player_minion)
        .add_systems(Update, player_minion_pickup);

        app.init_asset::<LevelAsset>()
            .init_asset_loader::<LevelAssetLoader>();

        app.add_systems(Startup, load_preview_scene);
        app.add_systems(Update, init_level);

        app.add_plugins(AudioPlugin);
    }
}

fn spawn_gameplay_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-30.0, 30.0, 100.0)
            .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

pub fn setup_player(mut commands: Commands) {
    let mut minion_st = MinionStorage::new();

    minion_st.add_minion(MinionKind::Doink);
    minion_st.add_minion(MinionKind::Doink);
    minion_st.add_minion(MinionKind::Doink);
    minion_st.add_minion(MinionKind::Doink);

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                ..default()
            },
            Collider::round_cylinder(0.9, 0.3, 0.2),
            KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                // Don’t allow climbing slopes larger than 45 degrees.
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: None,
                ..default()
            },
            minion_st,
        ))
        .with_children(|b| {
            b.spawn((SpatialBundle { ..default() }, PlayerCollector))
                .with_children(|b| {
                    b.spawn((
                        SpatialBundle {
                            transform: Transform::from_rotation(Quat::from_rotation_z(
                                std::f32::consts::FRAC_PI_2,
                            ))
                            .with_translation(Vec3::new(3.0, -1.0, 0.0)),
                            ..default()
                        },
                        Collider::cone(3.0, 4.5),
                        RigidBody::Fixed,
                        Sensor,
                    ));
                });
        });
}

pub fn mouse_tap(
    window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    rap_ctx: ResMut<RapierContext>,
    cam: Query<(&GlobalTransform, &Camera)>,
    mut gizmos: Gizmos,
    mut player: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut movement: ResMut<MovementInput>,
    mut dir: ResMut<PlayerDirection>,
    mut minion: ResMut<MinionInput>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let Some(pos) = window.cursor_position() else {
        return;
    };
    let Some((cam_tf, cam)) = cam
        .iter()
        .filter(|(_, cam)| matches!(cam.target, RenderTarget::Window(WindowRef::Primary)))
        .next()
    else {
        return;
    };
    let Some(cursor_ray) = cam.viewport_to_world(cam_tf, pos) else {
        return;
    };

    let Some((_, ray_hit)) = rap_ctx.cast_ray_and_get_normal(
        cursor_ray.origin,
        cursor_ray.direction.as_vec3(),
        1000.0,
        true,
        default(),
    ) else {
        return;
    };

    let Ok(hit_dir) = Dir3::new(ray_hit.normal) else {
        return;
    };

    gizmos.arrow(
        ray_hit.point + ray_hit.normal * 10.0,
        ray_hit.point,
        Color::linear_rgb(1.0, 0.0, 0.0),
    );

    gizmos.circle(
        ray_hit.point,
        hit_dir,
        3.0,
        Color::linear_rgb(1.0, 0.0, 0.0),
    );

    let Ok((player_tf, _, _)) = player.get_single_mut() else {
        return;
    };
    let walk_dir = (ray_hit.point - player_tf.translation).normalize_or_zero();

    if let Ok(walk_dir) = Dir3::new(walk_dir) {
        dir.0 = walk_dir;
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        movement.0 = walk_dir;
    }

    if mouse_buttons.just_pressed(MouseButton::Right) {
        minion.to_where = ray_hit.point;
        minion.want_to_throw = true;
    }
}

fn setup_physics(mut commands: Commands) {
    /*
     * Ground
     */
    let ground_size = 200.1;
    let ground_height = 0.1;

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -ground_height, 0.0)),
        Collider::cuboid(ground_size, ground_height, ground_size),
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
