use bevy_rapier3d::prelude::Collider;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use crate::tooling::editor::tilemap_mesh::RawMesh;

#[cfg_attr(not(target_family = "wasm"), derive(Serialize, Deserialize))]
pub struct LevelAsset {
    version: u32,
    data: LevelAssetData,
}
impl LevelAsset {
    pub fn new(data: LevelAssetData) -> Self {
        Self { version: 1, data }
    }

    // todo: custom loader
    pub fn load(path: &str) -> anyhow::Result<LevelAsset> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut data = vec![];
        reader.read_to_end(&mut data)?;

        if data.len() < 4 {
            anyhow::bail!("Not big enough");
        }

        let data = decompress_size_prepended(&data)?;
        let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        assert_eq!(version, 1);
        let data = bincode::deserialize::<LevelAssetData>(&data[4..])?;

        Ok(Self { version, data })
    }

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
