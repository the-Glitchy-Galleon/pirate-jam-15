use bevy::prelude::*;

#[derive(Component)]
pub struct TopDownCameraControls {
    pub target: Option<Entity>,
    pub offset: Vec3,
}

pub fn update(
    mut cams: Query<(&TopDownCameraControls, &mut Transform)>,
    targets: Query<&GlobalTransform>,
) {
    for (cam, mut ctf) in cams.iter_mut() {
        if let Some(target) = cam.target {
            if let Ok(ttf) = targets.get(target) {
                ctf.translation = ttf.translation() + cam.offset;
                ctf.look_at(ttf.translation(), Vec3::Y);
            }
        }
    }
}
