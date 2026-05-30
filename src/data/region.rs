//! Region resource (de)serialization: JSON schema plus a base64/RLE terrain
//! codec used to persist regions to disk.

use std::{fmt, fs, io, path::Path};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::data::{
    grid::{Grid, Vector},
    world::{Region, Terrain, TerrainType, WORLD_REGION_HEIGHT, WORLD_REGION_WIDTH},
};

/// Current region document version.
pub const REGION_VERSION: u32 = 1;
/// Terrain encoding identifier stored in the document.
pub const TERRAIN_ENCODING: &str = "base64-rle-u8-u16le";

const RUN_MAX: usize = u16::MAX as usize;

/// On-disk region document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionDocument {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub center: PointDoc,
    pub size: SizeDoc,
    pub terrain: TerrainDoc,
    #[serde(default)]
    pub entities: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointDoc {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeDoc {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainDoc {
    pub encoding: String,
    pub data: String,
}

/// Errors produced while loading or decoding a region document.
#[derive(Debug)]
pub enum RegionError {
    Io(io::Error),
    Json(serde_json::Error),
    Base64(base64::DecodeError),
    UnsupportedVersion(u32),
    UnsupportedEncoding(String),
    MalformedTriples(usize),
    UnknownTerrainId(u8),
    CellCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for RegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegionError::Io(error) => write!(f, "region io error: {error}"),
            RegionError::Json(error) => write!(f, "region json error: {error}"),
            RegionError::Base64(error) => write!(f, "region base64 error: {error}"),
            RegionError::UnsupportedVersion(version) => {
                write!(f, "unsupported region version: {version}")
            }
            RegionError::UnsupportedEncoding(encoding) => {
                write!(f, "unsupported terrain encoding: {encoding}")
            }
            RegionError::MalformedTriples(len) => {
                write!(f, "malformed terrain stream: {len} bytes is not a multiple of 3")
            }
            RegionError::UnknownTerrainId(id) => write!(f, "unknown terrain id: {id}"),
            RegionError::CellCountMismatch { expected, actual } => {
                write!(f, "decoded {actual} terrain cells, expected {expected}")
            }
        }
    }
}

impl std::error::Error for RegionError {}

impl From<io::Error> for RegionError {
    fn from(error: io::Error) -> Self {
        RegionError::Io(error)
    }
}

impl From<serde_json::Error> for RegionError {
    fn from(error: serde_json::Error) -> Self {
        RegionError::Json(error)
    }
}

/// Encodes terrain ids row-major as RLE triples (`id: u8`, `run_len: u16 LE`).
pub fn encode_rle(ids: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < ids.len() {
        let id = ids[index];
        let mut run = 1usize;
        while index + run < ids.len() && ids[index + run] == id {
            run += 1;
        }
        index += run;

        while run > 0 {
            let chunk = run.min(RUN_MAX);
            out.push(id);
            out.extend_from_slice(&(chunk as u16).to_le_bytes());
            run -= chunk;
        }
    }
    out
}

/// Decodes an RLE triple stream back into terrain ids.
pub fn decode_rle(bytes: &[u8]) -> Result<Vec<u8>, RegionError> {
    if !bytes.len().is_multiple_of(3) {
        return Err(RegionError::MalformedTriples(bytes.len()));
    }
    let mut ids = Vec::new();
    for triple in bytes.chunks_exact(3) {
        let id = triple[0];
        if TerrainType::from_id(id).is_none() {
            return Err(RegionError::UnknownTerrainId(id));
        }
        let run = u16::from_le_bytes([triple[1], triple[2]]) as usize;
        ids.extend(std::iter::repeat_n(id, run));
    }
    Ok(ids)
}

/// Encodes a terrain grid into the base64/RLE document string.
pub fn encode_terrain(terrain: &Grid<Terrain>) -> String {
    let mut ids = Vec::with_capacity(terrain.width() * terrain.height());
    for y in 0..terrain.height() {
        for x in 0..terrain.width() {
            let id = terrain
                .get(x, y)
                .map(|cell| cell.kind().id())
                .unwrap_or(0);
            ids.push(id);
        }
    }
    STANDARD.encode(encode_rle(&ids))
}

/// Decodes a base64/RLE document string into a terrain grid of the given size.
pub fn decode_terrain(
    data: &str,
    width: usize,
    height: usize,
) -> Result<Grid<Terrain>, RegionError> {
    let bytes = STANDARD.decode(data).map_err(RegionError::Base64)?;
    let ids = decode_rle(&bytes)?;
    let expected = width.saturating_mul(height);
    if ids.len() != expected {
        return Err(RegionError::CellCountMismatch {
            expected,
            actual: ids.len(),
        });
    }
    Ok(Grid::from_fn(width, height, |x, y| {
        let id = ids[y * width + x];
        Terrain::new(TerrainType::from_id(id).unwrap_or(TerrainType::Grass))
    }))
}

impl RegionDocument {
    /// Builds a document from a region centered at `center`.
    pub fn from_region(center: Vector, region: &Region) -> Self {
        let terrain = region.terrain();
        Self {
            version: REGION_VERSION,
            id: region.id().to_string(),
            name: region.name().to_string(),
            center: PointDoc {
                x: center.x,
                y: center.y,
            },
            size: SizeDoc {
                width: terrain.width() as u32,
                height: terrain.height() as u32,
            },
            terrain: TerrainDoc {
                encoding: TERRAIN_ENCODING.to_string(),
                data: encode_terrain(terrain),
            },
            entities: Vec::new(),
        }
    }

    /// Reconstructs the region center and region from this document.
    pub fn to_region(&self) -> Result<(Vector, Region), RegionError> {
        if self.version != REGION_VERSION {
            return Err(RegionError::UnsupportedVersion(self.version));
        }
        if self.terrain.encoding != TERRAIN_ENCODING {
            return Err(RegionError::UnsupportedEncoding(self.terrain.encoding.clone()));
        }
        let width = self.size.width as usize;
        let height = self.size.height as usize;
        let terrain = decode_terrain(&self.terrain.data, width, height)?;
        let center = Vector {
            x: self.center.x,
            y: self.center.y,
        };
        Ok((center, Region::new(self.id.clone(), self.name.clone(), terrain)))
    }
}

/// Builds the default all-grass `Bridgeport Outskirts` document.
pub fn default_bridgeport_outskirts() -> RegionDocument {
    let terrain = Grid::new(
        WORLD_REGION_WIDTH,
        WORLD_REGION_HEIGHT,
        Terrain::new(TerrainType::Grass),
    );
    let region = Region::new("bridgeport_outskirts", "Bridgeport Outskirts", terrain);
    RegionDocument::from_region(Vector { x: 0, y: 0 }, &region)
}

/// Loads a region document from `path`, creating an all-grass default when the
/// file is missing.
pub fn load_or_create(path: impl AsRef<Path>) -> Result<RegionDocument, RegionError> {
    let path = path.as_ref();
    if path.exists() {
        let contents = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        let document = default_bridgeport_outskirts();
        save_document(path, &document)?;
        Ok(document)
    }
}

/// Writes a region document to `path` as pretty JSON, creating parent dirs.
pub fn save_document(path: impl AsRef<Path>, document: &RegionDocument) -> Result<(), RegionError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(document)?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("frust-region-{}-{label}.json", std::process::id()))
    }

    #[test]
    fn rle_base64_round_trips_for_all_terrain_ids() {
        let width = 8;
        let height = 4;
        let grid = Grid::from_fn(width, height, |x, y| {
            let id = ((x + y * width) % super::super::world::TERRAIN_TYPES.len()) as u8;
            Terrain::new(TerrainType::from_id(id).unwrap())
        });

        let encoded = encode_terrain(&grid);
        let decoded = decode_terrain(&encoded, width, height).unwrap();
        assert_eq!(decoded, grid);
    }

    #[test]
    fn long_runs_split_over_u16_max() {
        let count = (RUN_MAX * 2) + 5;
        let ids = vec![3u8; count];
        let encoded = encode_rle(&ids);
        // 65535 + 65535 + 5 -> three triples.
        assert_eq!(encoded.len(), 9);
        assert_eq!(decode_rle(&encoded).unwrap(), ids);
    }

    #[test]
    fn decode_rejects_unknown_ids() {
        let bytes = [200u8, 1, 0];
        assert!(matches!(
            decode_rle(&bytes),
            Err(RegionError::UnknownTerrainId(200))
        ));
    }

    #[test]
    fn decode_rejects_malformed_triples() {
        let bytes = [0u8, 1];
        assert!(matches!(
            decode_rle(&bytes),
            Err(RegionError::MalformedTriples(2))
        ));
    }

    #[test]
    fn decode_rejects_wrong_cell_count() {
        let data = STANDARD.encode(encode_rle(&[0u8, 0, 0]));
        assert!(matches!(
            decode_terrain(&data, 4, 4),
            Err(RegionError::CellCountMismatch {
                expected: 16,
                actual: 3
            })
        ));
    }

    #[test]
    fn document_round_trips_through_region() {
        let document = default_bridgeport_outskirts();
        let (center, region) = document.to_region().unwrap();
        assert_eq!(center, Vector { x: 0, y: 0 });
        assert_eq!(region.id(), "bridgeport_outskirts");
        assert_eq!(region.name(), "Bridgeport Outskirts");
        let rebuilt = RegionDocument::from_region(center, &region);
        assert_eq!(rebuilt, document);
    }

    #[test]
    fn missing_file_creates_all_grass_default() {
        let path = unique_path("missing-creates-default");
        let _ = std::fs::remove_file(&path);

        let document = load_or_create(&path).unwrap();
        assert_eq!(document.name, "Bridgeport Outskirts");
        assert_eq!(document.id, "bridgeport_outskirts");
        assert_eq!(document.version, REGION_VERSION);
        assert!(path.exists());

        let (_, region) = document.to_region().unwrap();
        for y in 0..region.terrain().height() {
            for x in 0..region.terrain().width() {
                assert_eq!(region.terrain_at(x, y).unwrap().kind(), TerrainType::Grass);
            }
        }

        let _ = std::fs::remove_file(&path);
    }
}
