use crate::framework::prelude::*;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use bevy_rapier3d::prelude::Collider;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

#[derive(Asset, TypePath, Serialize, Deserialize)]
pub struct LevelAsset {
    version: u32,
    data: LevelAssetData,
}
impl LevelAsset {
    pub fn new(data: LevelAssetData) -> Self {
        Self { version: 1, data }
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

        let data = decompress_size_prepended(&data)?;
        let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        assert_eq!(version, 1);
        let data = bincode::deserialize::<LevelAssetData>(&data[4..])?;

        Ok(Self { version, data })
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let mut data = vec![];
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&bincode::serialize(&self.data)?);

        let data = compress_prepend_size(&data);

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
    pub ground_collider: Collider,
    pub ground_mesh: RawMesh,
    pub walls: Vec<WallData>,
}

#[derive(Serialize, Deserialize)]
pub struct WallData {
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
