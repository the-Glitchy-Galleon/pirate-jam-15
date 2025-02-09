use crate::{
    framework::{raw_mesh::RawMesh, tilemap::Tilemap},
    game::objects::definitions::ObjectDef,
};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(not(target_family = "wasm"))]
use {
    std::fs::File,
    std::io::{BufReader, BufWriter, Read, Write},
};

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct LevelAsset {
    version: u32,
    data: LevelAssetData,
}
impl LevelAsset {
    pub const CURRENT_VERSION: u32 = 4;
    pub fn new(data: LevelAssetData) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            data,
        }
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn read(path: &str) -> anyhow::Result<LevelAsset> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut data = vec![];
        reader.read_to_end(&mut data)?;
        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 4 {
            anyhow::bail!("Not big enough");
        }

        let data = lz4_flex::decompress_size_prepended(&data)?;
        let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        assert_eq!(version, Self::CURRENT_VERSION);
        let data = bincode::deserialize::<LevelAssetData>(&data[4..])?;

        Ok(Self { version, data })
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let mut data = vec![];
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&bincode::serialize(&self.data)?);

        let data = lz4_flex::compress_prepend_size(&data);

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&data)?;
        Ok(())
    }

    pub fn data(&self) -> &LevelAssetData {
        &self.data
    }
}

#[derive(Serialize, Deserialize)]
pub struct LevelAssetData {
    pub tilemap: Tilemap,
    pub objects: Vec<ObjectDef>,
    pub meshes: Vec<OrnamentalMesh>,
    pub baked_ground_mesh: RawMesh,
    pub baked_ground_collider: Collider,
    pub baked_walls: Vec<BakedWallData>,
}

#[derive(Serialize, Deserialize)]
pub struct BakedWallData {
    pub collider: Collider,
    pub mesh: RawMesh,
}

#[derive(Default)]
pub struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
    type Asset = LevelAsset;
    type Settings = ();
    type Error = anyhow::Error;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        LevelAsset::from_bytes(&bytes)
    }

    fn extensions(&self) -> &[&str] {
        &["level"]
    }
}

#[derive(Serialize, Deserialize)]
pub struct OrnamentalMesh {
    pub asset_path: String,
    pub position: Vec3,
    pub rotation: Vec3,
}
