mod png;
mod report;
mod storage;

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use report::InspectItem;
use uuid::Uuid;

pub use report::InspectReport;

const MAX_INPUT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_DECODED_PIXELS: u64 = 100_000_000;

#[derive(Debug)]
pub struct InspectRequest {
    pub input: PathBuf,
    pub candidate_dir: Option<PathBuf>,
}

pub fn inspect(request: InspectRequest) -> InspectReport {
    let session_id = Uuid::new_v4().to_string();
    let source = match fs::canonicalize(&request.input) {
        Ok(source) => source,
        Err(error) => {
            let (reason_code, diagnostic) = source_error(&error);
            return InspectReport::failed(
                session_id,
                InspectItem::failure(reason_code),
                diagnostic,
            );
        }
    };
    let identity = source.to_string_lossy().into_owned();
    let source_bytes = match read_source_bounded(&source) {
        Ok(bytes) => bytes,
        Err(SourceReadError::TooLarge) => {
            return InspectReport::failed(
                session_id,
                InspectItem::input_failure("input_byte_limit_exceeded", identity),
                "source exceeds the 512 MiB per-file input limit".to_owned(),
            );
        }
        Err(SourceReadError::Unreadable(error)) => {
            let (reason_code, diagnostic) = source_error(&error);
            return InspectReport::failed(
                session_id,
                InspectItem::input_failure(reason_code, identity),
                diagnostic,
            );
        }
    };
    let optimized = match png::optimize_losslessly(&source_bytes, MAX_DECODED_PIXELS) {
        Ok(optimized) => optimized,
        Err(error) => {
            let reason_code = error.reason_code();
            let item = InspectItem::png_failure(
                reason_code,
                identity,
                &source_bytes,
                error.recognized_as_png(),
            );
            let diagnostic = match reason_code {
                "decoded_pixel_limit_exceeded" => {
                    "PNG exceeds the 100 million pixels per-file decoded limit".to_owned()
                }
                "preservation_validation_failed" => {
                    format!("PNG preservation validation failed: {error}")
                }
                _ => format!("malformed PNG: {error}"),
            };
            return InspectReport::failed(session_id, item, diagnostic);
        }
    };

    let candidate_dir = match request.candidate_dir {
        Some(candidate_dir) => candidate_dir,
        None => match storage::default_candidate_dir() {
            Ok(candidate_dir) => candidate_dir,
            Err(error) => {
                let item = InspectItem::completed(&identity, &source_bytes, &optimized, None);
                return InspectReport::storage_failure(session_id, item, error);
            }
        },
    };
    let session_dir = candidate_dir.join(&session_id);
    let is_candidate = optimized.bytes.len() < source_bytes.len();
    let candidate_id = is_candidate.then(|| Uuid::new_v4().to_string());
    let item = InspectItem::completed(&identity, &source_bytes, &optimized, candidate_id.clone());

    if let Err(error) = storage::create_session_dir(&session_dir) {
        return InspectReport::storage_failure(session_id, item, error);
    }
    if let Some(candidate_id) = candidate_id.as_deref() {
        let candidate_path = session_dir.join(format!("{candidate_id}.png"));
        if let Err(error) = storage::store_candidate(&candidate_path, &optimized.bytes) {
            storage::clean_failed_session(&session_dir, Some(candidate_id));
            return InspectReport::storage_failure(session_id, item, error);
        }
    }

    let report = InspectReport::successful(session_id, item, is_candidate);
    let manifest = match report.manifest_bytes() {
        Ok(manifest) => manifest,
        Err(error) => {
            storage::clean_failed_session(&session_dir, candidate_id.as_deref());
            return report.into_storage_failure(error);
        }
    };
    if let Err(error) = storage::persist_manifest(&session_dir, &manifest) {
        storage::clean_failed_session(&session_dir, candidate_id.as_deref());
        return report.into_storage_failure(error);
    }
    report
}

pub fn invalid_invocation_report() -> InspectReport {
    InspectReport::invalid_invocation()
}

fn source_error(error: &io::Error) -> (&'static str, String) {
    if error.kind() == io::ErrorKind::NotFound {
        ("source_not_found", "source does not exist".to_owned())
    } else {
        (
            "source_unreadable",
            format!("source could not be read: {error}"),
        )
    }
}

enum SourceReadError {
    TooLarge,
    Unreadable(io::Error),
}

fn read_source_bounded(path: &Path) -> Result<Vec<u8>, SourceReadError> {
    let file = fs::File::open(path).map_err(SourceReadError::Unreadable)?;
    let metadata = file.metadata().map_err(SourceReadError::Unreadable)?;
    if !metadata.is_file() {
        return Err(SourceReadError::Unreadable(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input is not a regular file",
        )));
    }
    if metadata.len() > MAX_INPUT_BYTES {
        return Err(SourceReadError::TooLarge);
    }

    let mut bytes = Vec::new();
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(SourceReadError::Unreadable)?;
    if u64::try_from(bytes.len()).expect("usize fits into u64") > MAX_INPUT_BYTES {
        Err(SourceReadError::TooLarge)
    } else {
        Ok(bytes)
    }
}
