use crate::framework::tilemap::{Pnormal3, Tilemap, MAX_NUM_WALLS};

pub struct TilemapControls {
    max_elevation: u32,
    max_elevation_slope: u32,
}

impl TilemapControls {
    pub fn new(max_elevation: u32, max_elevation_slope: u32) -> Self {
        Self {
            max_elevation,
            max_elevation_slope,
        }
    }

    /// Raises tile elevation up to the configured cap
    pub fn raise_face_elevation(&self, tilemap: &mut Tilemap, fid: u32, amount: u32) {
        if !tilemap.face_grid().is_id_in_grid(fid) {
            return;
        }
        for vid in tilemap.face_id_to_vert_ids(fid) {
            let slope = tilemap
                .vert_neighbor_elevations(vid)
                .fold(u32::MAX, |acc, x| u32::min(acc, x));
            let vert = &mut tilemap.vert_data_mut()[vid as usize];
            vert.elevation = u32::min(
                vert.elevation + amount,
                u32::min(self.max_elevation, slope + self.max_elevation_slope),
            );
        }
    }
    pub fn lower_face_elevation(&self, tilemap: &mut Tilemap, fid: u32, amount: u32) {
        if !tilemap.face_grid().is_id_in_grid(fid) {
            return;
        }
        for vid in tilemap.face_id_to_vert_ids(fid) {
            let slope = tilemap
                .vert_neighbor_elevations(vid)
                .fold(u32::MIN, |acc, x| u32::max(acc, x));
            let vert = &mut tilemap.vert_data_mut()[vid as usize];
            vert.elevation = u32::max(
                u32::saturating_sub(vert.elevation, amount),
                u32::saturating_sub(slope, self.max_elevation_slope),
            );
        }
    }

    pub fn raise_wall_height(&self, tilemap: &mut Tilemap, fid: u32) {
        if let Some(face_data) = tilemap.face_data_mut().get_mut(fid as usize) {
            face_data.wall_height = u32::min(MAX_NUM_WALLS as u32, face_data.wall_height + 1);
        }
    }
    pub fn lower_wall_height(&self, tilemap: &mut Tilemap, fid: u32) {
        if let Some(face_data) = tilemap.face_data_mut().get_mut(fid as usize) {
            face_data.wall_height = u32::saturating_sub(face_data.wall_height, 1);
        }
    }

    pub fn paint_ground_face(&self, tilemap: &mut Tilemap, fid: u32, tid: u32) {
        if let Some(face_data) = tilemap.face_data_mut().get_mut(fid as usize) {
            face_data.tile_id = tid;
        }
    }
    pub fn paint_wall_face(
        &self,
        tilemap: &mut Tilemap,
        fid: u32,
        wnm: Pnormal3,
        height: u32,
        tid: u32,
    ) {
        if let Some(face_data) = tilemap.face_data_mut().get_mut(fid as usize) {
            match wnm {
                Pnormal3::NOO => face_data.wall_side_tile_ids[0 + (height * 4) as usize] = tid,
                Pnormal3::ONO => {}
                Pnormal3::OON => face_data.wall_side_tile_ids[1 + (height * 4) as usize] = tid,
                Pnormal3::POO => face_data.wall_side_tile_ids[2 + (height * 4) as usize] = tid,
                Pnormal3::OPO => face_data.wall_top_tile_id = tid,
                Pnormal3::OOP => face_data.wall_side_tile_ids[3 + (height * 4) as usize] = tid,
            }
        }
    }
}
