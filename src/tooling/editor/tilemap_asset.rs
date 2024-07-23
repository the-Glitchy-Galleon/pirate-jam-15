use super::object_def_widget::{ObjectDefBuilder, Rot8};
use super::tilemap::{Pnormal2, Tilemap};
use bevy::asset::io::Reader;
use bevy::asset::AsyncReadExt;
use bevy::{
    asset::{AssetLoader, LoadContext},
    prelude::*,
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufWriter, Read};
use std::path::Path;

pub struct TilemapAssetPlugin;

impl Plugin for TilemapAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Tilemap>()
            .register_asset_loader(TilemapLoader);
    }
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct TilemapRon {
    pub version: u32,
    pub tilemap: Tilemap,
    pub objects: Vec<ObjectDefBuilder>,
    pub meshes: Vec<OrnamentalMeshBuilder>,
    // dependencies: Vec<String>,
    // embedded_dependencies: Vec<String>,
    // dependencies_with_settings: Vec<(String, ())>,
}
impl TilemapRon {
    pub const CURRENT_VERSION: u32 = 3;

    pub fn new(
        tilemap: Tilemap,
        objects: Vec<ObjectDefBuilder>,
        meshes: Vec<OrnamentalMeshBuilder>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            tilemap,
            objects,
            meshes,
        }
    }

    pub fn read<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mut bytes = Vec::new();
        let mut file = OpenOptions::new().read(true).open(path)?;
        // let mut reader = BufReader::new(file);
        file.read_to_end(&mut bytes)?;
        let result: TilemapRon = ron::de::from_bytes(&bytes)?;
        assert_eq!(result.version, Self::CURRENT_VERSION);
        Ok(result)
    }
    pub fn write<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let file = OpenOptions::new().create(true).write(true).open(path)?;
        let writer = BufWriter::new(file);
        ron::ser::to_writer_pretty(writer, self, PrettyConfig::new())?;
        Ok(())
    }
}

#[derive(Default)]
struct TilemapLoader;

impl AssetLoader for TilemapLoader {
    type Asset = Tilemap;
    type Settings = ();
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let ron: TilemapRon = ron::de::from_bytes(&bytes)?;
        Ok(ron.tilemap)
    }
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct GroundOrnamentalMesh {
    pub mesh_path: String,
    pub coord: UVec2,
    pub rot: Rot8,
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct WallSideOrnamentalMesh {
    pub mesh_path: String,
    pub side: Pnormal2,
    pub height: u32,
    pub rot: Rot8,
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct WallTopOrnamentalMesh {
    pub mesh_path: String,
    pub coord: UVec2,
    pub rot: Rot8,
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub enum OrnamentalMeshBuilder {
    Ground(GroundOrnamentalMesh),
    Side(WallSideOrnamentalMesh),
    Top(WallTopOrnamentalMesh),
}
