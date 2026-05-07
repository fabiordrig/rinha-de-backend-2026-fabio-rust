use std::{collections::HashMap, fs, io::Read, path::Path};

use flate2::read::GzDecoder;

use crate::{
    index::{load_index_file_mmap, MappedIndex},
    types::{Normalization, ReferenceLabel, ReferenceRecord},
};

#[derive(Debug)]
pub enum LoadedReferenceSource {
    Owned(Vec<ReferenceRecord>),
    Mapped(MappedIndex),
}

impl LoadedReferenceSource {
    pub fn len(&self) -> usize {
        match self {
            Self::Owned(records) => records.len(),
            Self::Mapped(index) => index.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn vector(&self, index: usize) -> Option<&[f64; 14]> {
        match self {
            Self::Owned(records) => records.get(index).map(|record| &record.vector),
            Self::Mapped(index_file) => index_file.vector(index),
        }
    }

    pub fn label(&self, index: usize) -> Option<ReferenceLabel> {
        match self {
            Self::Owned(records) => records.get(index).map(|record| record.label),
            Self::Mapped(index_file) => index_file.label(index),
        }
    }
}

#[derive(Debug)]
pub struct LoadedResources {
    pub normalization: Normalization,
    pub mcc_risk: HashMap<String, f64>,
    pub references: LoadedReferenceSource,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadResourcesError {
    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    ParseJson {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("missing references file in {0}")]
    MissingReferences(String),
}

pub fn load_resources_from_dir(path: &Path) -> Result<LoadedResources, LoadResourcesError> {
    let normalization_path = path.join("normalization.json");
    let mcc_risk_path = path.join("mcc_risk.json");
    let references_binary_path = path.join("references.bin");
    let references_gzip_path = path.join("references.json.gz");
    let references_json_path = path.join("references.json");

    let normalization: Normalization = parse_json_file(&normalization_path)?;
    let mcc_risk: HashMap<String, f64> = parse_json_file(&mcc_risk_path)?;

    let references = if references_binary_path.exists() {
        LoadedReferenceSource::Mapped(
            load_index_file_mmap(&references_binary_path).map_err(|source| {
                LoadResourcesError::ReadFile {
                    path: references_binary_path.display().to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidData, source.to_string()),
                }
            })?,
        )
    } else if references_gzip_path.exists() {
        LoadedReferenceSource::Owned(parse_gzip_json_file(&references_gzip_path)?)
    } else if references_json_path.exists() {
        LoadedReferenceSource::Owned(parse_json_file(&references_json_path)?)
    } else {
        return Err(LoadResourcesError::MissingReferences(
            path.display().to_string(),
        ));
    };

    Ok(LoadedResources {
        normalization,
        mcc_risk,
        references,
    })
}

fn parse_json_file<T>(path: &Path) -> Result<T, LoadResourcesError>
where
    T: serde::de::DeserializeOwned,
{
    let content = fs::read_to_string(path).map_err(|source| LoadResourcesError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;

    serde_json::from_str(&content).map_err(|source| LoadResourcesError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn parse_gzip_json_file<T>(path: &Path) -> Result<T, LoadResourcesError>
where
    T: serde::de::DeserializeOwned,
{
    let file = fs::File::open(path).map_err(|source| LoadResourcesError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;

    let mut decoder = GzDecoder::new(file);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .map_err(|source| LoadResourcesError::ReadFile {
            path: path.display().to_string(),
            source,
        })?;

    serde_json::from_str(&content).map_err(|source| LoadResourcesError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}
