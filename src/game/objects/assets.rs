use bevy::prelude::*;

#[derive(Resource)]
pub struct GameObjectAssets {
    pub camera_wall_mount: Handle<Mesh>,
    pub camera_rotating_mesh: Handle<Mesh>,
    pub camera_material: Handle<StandardMaterial>,
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

        Self {
            camera_wall_mount,
            camera_rotating_mesh,
            camera_material,
        }
    }
}
