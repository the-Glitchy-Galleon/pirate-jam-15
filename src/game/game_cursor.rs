use super::{
    collision_groups::{GROUND_GROUP, TARGET_GROUP, WALL_GROUP},
    level::GroundTag,
    minion::MinionTarget,
};
use crate::{
    framework::logical_cursor::{self, LogicalCursor},
    game::common::PrimaryCamera,
};
use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    time::Real,
};
use bevy_rapier3d::prelude::*;
use std::f32::consts::{PI, TAU};

pub struct GameCursorPlugin;

impl Plugin for GameCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<DecalMaterial> {
            prepass_enabled: false,
            ..default()
        })
        .insert_resource(ShowCursorMode::None)
        .init_resource::<GameCursor>()
        .add_systems(
            Startup,
            (
                setup_ground_cursor_decal,
                setup_point_cursor_decal,
                setup_target_cursor_hud,
            ),
        )
        .add_systems(PreUpdate, update_game_cursor.after(logical_cursor::update))
        .add_systems(
            Update,
            (
                update_show_cursor_mode,
                update_ground_cursor_decal.after(update_show_cursor_mode),
                update_point_cursor_decal.after(update_show_cursor_mode),
                update_target_cursor_hud.after(update_show_cursor_mode),
            ),
        );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DecalMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material for DecalMaterial {
    // fn vertex_shader() -> ShaderRef {
    //     "shader/decal.wgsl".into()
    // }

    fn fragment_shader() -> ShaderRef {
        "shader/decal.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Resource, Default)]
pub struct GameCursor {
    pub hit: Option<GameCursorHit>,
    pub lock: Option<Entity>,
}
pub struct GameCursorHit {
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
}

pub fn update_game_cursor(
    mut cursor: ResMut<GameCursor>,
    camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
    lcursor: Res<LogicalCursor>,
    rapier: ResMut<RapierContext>,
    targets: Query<Entity, With<MinionTarget>>,
) {
    cursor.hit = None;
    cursor.lock = None;
    let (camera, camera_gx) = camera.single();

    let Some(cursor_position) = lcursor.position else {
        return;
    };
    let Some(ray) = camera.viewport_to_world(camera_gx, cursor_position) else {
        return;
    };
    let Some((entity, hit)) = rapier.cast_ray_and_get_normal(
        ray.origin,
        ray.direction.as_vec3(),
        bevy_rapier3d::math::Real::INFINITY,
        true,
        QueryFilter {
            groups: Some(CollisionGroups::new(
                Group::all(),
                GROUND_GROUP | WALL_GROUP | TARGET_GROUP,
            )),
            ..default()
        },
    ) else {
        return;
    };
    cursor.hit = Some(GameCursorHit {
        entity,
        point: hit.point.into(),
        normal: hit.normal,
    });

    if targets.contains(entity) {
        cursor.lock = Some(entity);
    }
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum ShowCursorMode {
    None,
    Point,
    Ground,
    Target,
}

pub fn update_show_cursor_mode(
    cursor: Res<GameCursor>,
    ground: Query<(), With<GroundTag>>,
    mut mode: ResMut<ShowCursorMode>,
) {
    *mode = if cursor.lock.is_some() {
        ShowCursorMode::Target
    } else {
        if let Some(hit) = &cursor.hit {
            if ground.get(hit.entity).is_ok() {
                ShowCursorMode::Ground
            } else {
                ShowCursorMode::Point
            }
        } else {
            ShowCursorMode::None
        }
    };
}

#[derive(Component)]
pub struct GroundCursorDecal;

pub fn setup_ground_cursor_decal(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<DecalMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let texture = ass.load("ui/ground_cursor.png");
    let mesh = meshes.add(
        Rectangle::from_size(Vec2::splat(2.0))
            .mesh()
            .build()
            .rotated_by(Quat::from_rotation_arc(Vec3::Z, Vec3::Y))
            .with_generated_tangents()
            .unwrap(),
    );
    let root = (
        SpatialBundle {
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        GroundCursorDecal,
    );
    let decal = (
        NotShadowCaster,
        SpatialBundle {
            transform: Transform::IDENTITY.with_translation(Vec3::Y * 0.1),
            visibility: Visibility::Inherited,
            ..Default::default()
        },
        mesh,
        materials.add(DecalMaterial {
            texture: texture.clone(),
            base_color: LinearRgba::RED,
        }),
    );

    cmd.spawn(root).with_children(|cmd| {
        cmd.spawn(decal);
    });
}

pub fn update_ground_cursor_decal(
    mut decal: Query<(&mut Transform, &mut Visibility), With<GroundCursorDecal>>,
    cursor: Res<GameCursor>,
    time: Res<Time<Real>>,
    mode: Res<ShowCursorMode>,
) {
    let (mut decal_tx, mut vis) = decal.single_mut();

    *vis = Visibility::Hidden;
    if *mode != ShowCursorMode::Ground {
        return;
    }
    let Some(hit) = &cursor.hit else {
        return;
    };

    *vis = Visibility::Visible;
    decal_tx.translation = hit.point;
    decal_tx.rotation = Quat::from_axis_angle(Vec3::Y, (time.elapsed_seconds() * PI) % TAU)
}

#[derive(Component)]
pub struct PointCursorDecal;

pub fn setup_point_cursor_decal(
    mut cmd: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<DecalMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let texture = ass.load("ui/point_cursor.png");
    let mesh = meshes.add(
        Rectangle::from_size(Vec2::splat(0.25))
            .mesh()
            .build()
            .rotated_by(Quat::from_axis_angle(Vec3::X, PI))
            .with_generated_tangents()
            .unwrap(),
    );
    let root = (
        SpatialBundle {
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        PointCursorDecal,
    );
    let decal = (
        NotShadowCaster,
        SpatialBundle {
            transform: Transform::IDENTITY.with_translation(Vec3::NEG_Z * 0.05),
            visibility: Visibility::Inherited,
            ..Default::default()
        },
        mesh,
        materials.add(DecalMaterial {
            texture: texture.clone(),
            base_color: LinearRgba::RED,
        }),
    );
    cmd.spawn(root).with_children(|cmd| {
        cmd.spawn(decal);
    });
}

pub fn update_point_cursor_decal(
    mut decal: Query<(&mut Transform, &mut Visibility), With<PointCursorDecal>>,
    cursor: Res<GameCursor>,
    mode: Res<ShowCursorMode>,
) {
    let (mut decal_tx, mut vis) = decal.single_mut();
    *vis = Visibility::Hidden;
    if *mode != ShowCursorMode::Point {
        return;
    }
    let Some(hit) = &cursor.hit else {
        return;
    };

    *vis = Visibility::Visible;
    decal_tx.translation = hit.point;
    decal_tx.look_at(hit.point + hit.normal, Vec3::Y);
}

#[derive(Component)]
pub struct TargetCursorHud;

pub fn setup_target_cursor_hud(mut cmd: Commands, ass: Res<AssetServer>) {
    let image = ass.load("ui/target_cursor.png");
    let root = (
        TargetCursorHud,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                overflow: Overflow::visible(),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..Default::default()
        },
    );
    let image = (ImageBundle {
        image: UiImage::new(image.clone()),
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(-64.0),
            left: Val::Px(-64.0),
            min_width: Val::Px(128.0),
            min_height: Val::Px(128.0),
            ..Default::default()
        },
        ..Default::default()
    },);

    cmd.spawn(root).with_children(|cmd| {
        cmd.spawn(image);
    });
}

pub fn update_target_cursor_hud(
    mut hud: Query<(&mut Style, &mut Transform, &mut Visibility), With<TargetCursorHud>>,
    target: Query<&GlobalTransform>,
    camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
    cursor: Res<GameCursor>,
    mode: Res<ShowCursorMode>,
    time: Res<Time<Real>>,
) {
    let (mut style, mut tx, mut vis) = hud.single_mut();
    *vis = Visibility::Hidden;
    if *mode != ShowCursorMode::Target {
        return;
    }
    let (camera, camera_gx) = camera.single();
    let Some(lock) = cursor.lock else {
        return;
    };
    let Ok(target) = target.get(lock) else {
        return;
    };
    if let Some(pos) = camera.world_to_viewport(camera_gx, target.translation()) {
        *vis = Visibility::Visible;
        style.left = Val::Px(pos.x);
        style.top = Val::Px(pos.y);
    }
    tx.rotation = Quat::from_axis_angle(Vec3::Z, (time.elapsed_seconds() * PI) % TAU);
    tx.scale = Vec3::splat(0.8 + (time.elapsed_seconds() * PI).sin() * 0.125);
}
