use bevy::prelude::*;

#[derive(Resource)]
pub struct GameObjectAssets {
    pub camera_mesh: Handle<Mesh>,
    pub camera_material: Handle<StandardMaterial>,
}

impl FromWorld for GameObjectAssets {
    fn from_world(world: &mut World) -> Self {
        let camera_mesh = {
            let handle = {
                // let ass = world.resource::<AssetServer>();
                Cuboid::new(0.25, 0.25, 0.25)
            };
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(handle)
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
            camera_mesh,
            camera_material,
        }
    }
}
