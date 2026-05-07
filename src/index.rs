use std::{fmt, fs, mem::size_of, path::Path};

use bytemuck::{cast_slice, Pod, Zeroable};
use memmap2::MmapOptions;

use crate::{
    resources::{load_resources_from_dir, LoadResourcesError},
    types::{ReferenceLabel, ReferenceRecord},
};

const INDEX_MAGIC: &[u8; 8] = b"RINHIDX1";
const HEADER_LEN: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct RawReferenceRecord {
    pub(crate) vector: [f64; 14],
    pub(crate) label: u8,
    pub(crate) _padding: [u8; 7],
}

pub struct MappedIndex {
    mmap: memmap2::Mmap,
    record_count: usize,
}

impl fmt::Debug for MappedIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MappedIndex")
            .field("record_count", &self.record_count)
            .finish()
    }
}

impl MappedIndex {
    pub fn len(&self) -> usize {
        self.record_count
    }

    pub fn is_empty(&self) -> bool {
        self.record_count == 0
    }

    pub fn record(&self, index: usize) -> Option<ReferenceRecord> {
        self.raw_records().get(index).map(|raw| ReferenceRecord {
            vector: raw.vector,
            label: byte_to_label(raw.label),
        })
    }

    pub fn vector(&self, index: usize) -> Option<&[f64; 14]> {
        self.raw_records().get(index).map(|raw| &raw.vector)
    }

    pub fn label(&self, index: usize) -> Option<ReferenceLabel> {
        self.raw_records().get(index).map(|raw| byte_to_label(raw.label))
    }

    pub fn to_vec(&self) -> Vec<ReferenceRecord> {
        self.raw_records()
            .iter()
            .map(|raw| ReferenceRecord {
                vector: raw.vector,
                label: byte_to_label(raw.label),
            })
            .collect()
    }

    fn raw_records(&self) -> &[RawReferenceRecord] {
        let payload = &self.mmap[HEADER_LEN..];
        bytemuck::cast_slice(payload)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error(transparent)]
    LoadResources(#[from] LoadResourcesError),
    #[error("failed to write index {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read index {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to memory-map index {path}: {source}")]
    Map {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid index header in {0}")]
    InvalidHeader(String),
    #[error("invalid index length in {0}")]
    InvalidLength(String),
}

pub fn build_index_from_resources_dir(
    resources_dir: &Path,
    output_path: &Path,
) -> Result<(), IndexError> {
    let resources = load_resources_from_dir(resources_dir)?;

    let raw_records = (0..resources.references.len())
        .map(|index| RawReferenceRecord {
            vector: *resources.references.vector(index).unwrap(),
            label: label_to_byte(resources.references.label(index).unwrap()),
            _padding: [0; 7],
        })
        .collect::<Vec<_>>();

    let mut bytes = Vec::with_capacity(HEADER_LEN + raw_records.len() * size_of::<RawReferenceRecord>());
    bytes.extend_from_slice(INDEX_MAGIC);
    bytes.extend_from_slice(&(raw_records.len() as u64).to_le_bytes());
    bytes.extend_from_slice(cast_slice(&raw_records));

    fs::write(output_path, bytes).map_err(|source| IndexError::Write {
        path: output_path.display().to_string(),
        source,
    })?;

    Ok(())
}

pub fn load_index_file(path: &Path) -> Result<Vec<ReferenceRecord>, IndexError> {
    Ok(load_index_file_mmap(path)?.to_vec())
}

pub fn load_index_file_mmap(path: &Path) -> Result<MappedIndex, IndexError> {
    let file = fs::File::open(path).map_err(|source| IndexError::Read {
        path: path.display().to_string(),
        source,
    })?;

    let mmap = unsafe { MmapOptions::new().map(&file) }.map_err(|source| IndexError::Map {
        path: path.display().to_string(),
        source,
    })?;

    let record_count = validate_index_layout(path, &mmap)?;

    Ok(MappedIndex { mmap, record_count })
}

pub fn index_header_bytes(record_count: usize) -> [u8; HEADER_LEN] {
    let mut header = [0_u8; HEADER_LEN];
    header[..8].copy_from_slice(INDEX_MAGIC);
    header[8..16].copy_from_slice(&(record_count as u64).to_le_bytes());
    header
}

fn validate_index_layout(path: &Path, bytes: &[u8]) -> Result<usize, IndexError> {
    if bytes.len() < HEADER_LEN || &bytes[..8] != INDEX_MAGIC {
        return Err(IndexError::InvalidHeader(path.display().to_string()));
    }

    let count = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let payload = &bytes[16..];
    let expected_len = count * size_of::<RawReferenceRecord>();

    if payload.len() != expected_len {
        return Err(IndexError::InvalidLength(path.display().to_string()));
    }

    bytemuck::try_cast_slice::<u8, RawReferenceRecord>(payload)
        .map_err(|_| IndexError::InvalidLength(path.display().to_string()))?;

    Ok(count)
}

fn label_to_byte(label: ReferenceLabel) -> u8 {
    match label {
        ReferenceLabel::Fraud => 1,
        ReferenceLabel::Legit => 0,
    }
}

fn byte_to_label(value: u8) -> ReferenceLabel {
    match value {
        1 => ReferenceLabel::Fraud,
        _ => ReferenceLabel::Legit,
    }
}
