mod png;

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const SCHEMA_VERSION: u32 = 1;
const MANIFEST_VERSION: u32 = 1;

#[derive(Debug)]
pub struct InspectRequest {
    pub input: PathBuf,
    pub candidate_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub struct InspectReport {
    schema_version: u32,
    command: &'static str,
    session_id: String,
    summary: Summary,
    items: Vec<InspectItem>,
}

#[derive(Debug, Serialize)]
struct Manifest<'a> {
    manifest_version: u32,
    session_id: &'a str,
    items: &'a [InspectItem],
}

#[derive(Debug, Serialize)]
struct Summary {
    candidate: u32,
    unchanged: u32,
    skipped: u32,
    failed: u32,
}

#[derive(Debug, Serialize)]
struct InspectItem {
    outcome: Outcome,
    reason_code: &'static str,
    source: String,
    destination: String,
    format: &'static str,
    source_hash: String,
    destination_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_hash: Option<String>,
    original_bytes: u64,
    candidate_bytes: u64,
    original_dimensions: Dimensions,
    candidate_dimensions: Dimensions,
    #[serde(skip_serializing_if = "Option::is_none")]
    savings_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    savings_percent: Option<f64>,
    resolved_policy: ResolvedPolicy,
    metadata_policy: MetadataPolicy,
    operations: [&'static str; 1],
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Candidate,
    Unchanged,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Dimensions {
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize)]
struct ResolvedPolicy {
    preset: &'static str,
    policy_id: &'static str,
}

#[derive(Debug, Serialize)]
struct MetadataPolicy {
    mode: &'static str,
    supported_classes: [&'static str; 5],
    present_classes: Vec<&'static str>,
}

pub fn inspect(request: InspectRequest) -> Result<InspectReport, InspectError> {
    let source = fs::canonicalize(&request.input)?;
    let source_bytes = fs::read(&source)?;
    let optimized = png::optimize_losslessly(&source_bytes)?;
    let session_id = Uuid::new_v4().to_string();
    let candidate_dir = match request.candidate_dir {
        Some(candidate_dir) => candidate_dir,
        None => default_candidate_dir()?,
    };
    let session_dir = candidate_dir.join(&session_id);
    fs::create_dir_all(&session_dir)?;

    let is_candidate = optimized.bytes.len() < source_bytes.len();
    let candidate_id = is_candidate.then(|| Uuid::new_v4().to_string());
    if let Some(candidate_id) = candidate_id.as_deref() {
        let candidate_path = session_dir.join(format!("{candidate_id}.png"));
        create_immutable_file(&candidate_path, &optimized.bytes)?;
    }

    let original_bytes = u64::try_from(source_bytes.len()).expect("usize fits into u64");
    let candidate_bytes = u64::try_from(optimized.bytes.len()).expect("usize fits into u64");
    let savings_bytes = is_candidate.then_some(original_bytes - candidate_bytes);
    let savings_percent = savings_bytes.map(|saved| saved as f64 * 100.0 / original_bytes as f64);
    let identity = source.to_string_lossy().into_owned();
    let item = InspectItem {
        outcome: if is_candidate {
            Outcome::Candidate
        } else {
            Outcome::Unchanged
        },
        reason_code: if is_candidate {
            "smaller_lossless_candidate"
        } else {
            "not_strictly_smaller"
        },
        source: identity.clone(),
        destination: identity,
        format: "png",
        source_hash: sha256(&source_bytes),
        destination_hash: sha256(&source_bytes),
        candidate_hash: is_candidate.then(|| sha256(&optimized.bytes)),
        original_bytes,
        candidate_bytes,
        original_dimensions: Dimensions {
            width: optimized.width,
            height: optimized.height,
        },
        candidate_dimensions: Dimensions {
            width: optimized.width,
            height: optimized.height,
        },
        savings_bytes,
        savings_percent,
        resolved_policy: ResolvedPolicy {
            preset: "lossless",
            policy_id: "png-lossless-v1",
        },
        metadata_policy: MetadataPolicy {
            mode: "preserve",
            supported_classes: [
                "color_appearance",
                "exif",
                "physical_dimensions",
                "text",
                "unknown_ancillary",
            ],
            present_classes: optimized.present_metadata_classes,
        },
        operations: ["lossless_compression"],
        candidate_id,
    };
    let report = InspectReport {
        schema_version: SCHEMA_VERSION,
        command: "inspect",
        session_id,
        summary: Summary {
            candidate: u32::from(is_candidate),
            unchanged: u32::from(!is_candidate),
            skipped: 0,
            failed: 0,
        },
        items: vec![item],
    };
    persist_manifest(&session_dir, &report)?;
    Ok(report)
}

#[cfg(target_os = "macos")]
fn default_candidate_dir() -> Result<PathBuf, InspectError> {
    Ok(home_dir()?.join("Library/Caches/image-optimizer"))
}

#[cfg(target_os = "linux")]
fn default_candidate_dir() -> Result<PathBuf, InspectError> {
    let xdg_cache_home = std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());
    let cache_home = match xdg_cache_home {
        Some(cache_home) => cache_home,
        None => home_dir()?.join(".cache"),
    };
    Ok(cache_home.join("image-optimizer"))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn default_candidate_dir() -> Result<PathBuf, InspectError> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "default candidate storage is supported only on macOS and Linux",
    )
    .into())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn home_dir() -> Result<PathBuf, InspectError> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; use --candidate-dir to select candidate storage",
        )
        .into()
    })
}

fn persist_manifest(session_dir: &Path, report: &InspectReport) -> Result<(), InspectError> {
    let manifest = Manifest {
        manifest_version: MANIFEST_VERSION,
        session_id: &report.session_id,
        items: &report.items,
    };
    let bytes = serde_json::to_vec_pretty(&manifest)?;
    create_immutable_file(&session_dir.join("manifest.json"), &bytes)
}

fn create_immutable_file(path: &Path, bytes: &[u8]) -> Result<(), InspectError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions)?;
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[derive(Debug)]
pub struct InspectError {
    detail: String,
}

impl std::fmt::Display for InspectError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for InspectError {}

impl From<io::Error> for InspectError {
    fn from(error: io::Error) -> Self {
        Self {
            detail: format!("filesystem error: {error}"),
        }
    }
}

impl From<png::PngOptimizationError> for InspectError {
    fn from(error: png::PngOptimizationError) -> Self {
        Self {
            detail: format!("PNG optimization failed: {error}"),
        }
    }
}

impl From<serde_json::Error> for InspectError {
    fn from(error: serde_json::Error) -> Self {
        Self {
            detail: format!("manifest serialization failed: {error}"),
        }
    }
}
