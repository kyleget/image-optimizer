use serde::Serialize;
use sha2::{Digest, Sha256};

use super::png::OptimizedPng;

const SCHEMA_VERSION: u32 = 1;
const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub struct InspectReport {
    schema_version: u32,
    command: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    summary: Summary,
    items: Vec<InspectItem>,
    #[serde(skip)]
    diagnostic: Option<String>,
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
pub(super) struct InspectItem {
    outcome: Outcome,
    reason_code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    destination_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    original_dimensions: Option<Dimensions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_dimensions: Option<Dimensions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    savings_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    savings_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_policy: Option<ResolvedPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata_policy: Option<MetadataPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operations: Option<[&'static str; 1]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Candidate,
    Unchanged,
    Failed,
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

impl InspectReport {
    pub(super) fn successful(session_id: String, item: InspectItem, is_candidate: bool) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command: "inspect",
            session_id: Some(session_id),
            summary: Summary {
                candidate: u32::from(is_candidate),
                unchanged: u32::from(!is_candidate),
                skipped: 0,
                failed: 0,
            },
            items: vec![item],
            diagnostic: None,
        }
    }

    pub(super) fn failed(session_id: String, item: InspectItem, diagnostic: String) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command: "inspect",
            session_id: Some(session_id),
            summary: Summary {
                candidate: 0,
                unchanged: 0,
                skipped: 0,
                failed: 1,
            },
            items: vec![item],
            diagnostic: Some(diagnostic),
        }
    }

    pub(super) fn storage_failure(
        session_id: String,
        item: InspectItem,
        error: impl std::fmt::Display,
    ) -> Self {
        Self::failed(
            session_id,
            item.into_storage_failure(),
            format!("candidate storage failed: {error}"),
        )
    }

    pub(super) fn into_storage_failure(self, error: impl std::fmt::Display) -> Self {
        let session_id = self
            .session_id
            .expect("completed inspection has a session identifier");
        let item = self.items.into_iter().next().expect("one inspect item");
        Self::storage_failure(session_id, item, error)
    }

    pub(super) fn invalid_invocation() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command: "inspect",
            session_id: None,
            summary: Summary {
                candidate: 0,
                unchanged: 0,
                skipped: 0,
                failed: 1,
            },
            items: vec![InspectItem::failure("invalid_invocation")],
            diagnostic: None,
        }
    }

    pub(super) fn manifest_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let manifest = Manifest {
            manifest_version: MANIFEST_VERSION,
            session_id: self
                .session_id
                .as_deref()
                .expect("persisted inspection has a session identifier"),
            items: &self.items,
        };
        serde_json::to_vec_pretty(&manifest)
    }

    pub fn has_failures(&self) -> bool {
        self.summary.failed != 0
    }

    pub fn diagnostic(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }
}

impl InspectItem {
    pub(super) fn failure(reason_code: &'static str) -> Self {
        Self {
            outcome: Outcome::Failed,
            reason_code,
            source: None,
            destination: None,
            format: None,
            source_hash: None,
            destination_hash: None,
            candidate_hash: None,
            original_bytes: None,
            candidate_bytes: None,
            original_dimensions: None,
            candidate_dimensions: None,
            savings_bytes: None,
            savings_percent: None,
            resolved_policy: None,
            metadata_policy: None,
            operations: None,
            candidate_id: None,
        }
    }

    pub(super) fn input_failure(reason_code: &'static str, identity: String) -> Self {
        let mut item = Self::failure(reason_code);
        item.source = Some(identity);
        item
    }

    pub(super) fn png_failure(
        reason_code: &'static str,
        identity: String,
        source_bytes: &[u8],
        recognized_as_png: bool,
    ) -> Self {
        let source_hash = sha256(source_bytes);
        let mut item = Self::failure(reason_code);
        item.source = Some(identity.clone());
        item.destination = Some(identity);
        item.format = recognized_as_png.then_some("png");
        item.source_hash = Some(source_hash.clone());
        item.destination_hash = Some(source_hash);
        item.original_bytes = Some(u64::try_from(source_bytes.len()).expect("usize fits into u64"));
        item
    }

    pub(super) fn completed(
        identity: &str,
        source_bytes: &[u8],
        optimized: &OptimizedPng,
        candidate_id: Option<String>,
    ) -> Self {
        let is_candidate = candidate_id.is_some();
        let original_bytes = u64::try_from(source_bytes.len()).expect("usize fits into u64");
        let candidate_bytes = u64::try_from(optimized.bytes.len()).expect("usize fits into u64");
        let savings_bytes = is_candidate.then_some(original_bytes - candidate_bytes);
        let savings_percent =
            savings_bytes.map(|saved| saved as f64 * 100.0 / original_bytes as f64);
        Self {
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
            source: Some(identity.to_owned()),
            destination: Some(identity.to_owned()),
            format: Some("png"),
            source_hash: Some(sha256(source_bytes)),
            destination_hash: Some(sha256(source_bytes)),
            candidate_hash: is_candidate.then(|| sha256(&optimized.bytes)),
            original_bytes: Some(original_bytes),
            candidate_bytes: Some(candidate_bytes),
            original_dimensions: Some(Dimensions {
                width: optimized.width,
                height: optimized.height,
            }),
            candidate_dimensions: Some(Dimensions {
                width: optimized.width,
                height: optimized.height,
            }),
            savings_bytes,
            savings_percent,
            resolved_policy: Some(ResolvedPolicy {
                preset: "lossless",
                policy_id: "png-lossless-v1",
            }),
            metadata_policy: Some(MetadataPolicy {
                mode: "preserve",
                supported_classes: [
                    "color_appearance",
                    "exif",
                    "physical_dimensions",
                    "text",
                    "unknown_ancillary",
                ],
                present_classes: optimized.present_metadata_classes.clone(),
            }),
            operations: Some(["lossless_compression"]),
            candidate_id,
        }
    }

    fn into_storage_failure(mut self) -> Self {
        self.outcome = Outcome::Failed;
        self.reason_code = "candidate_storage_failed";
        self.candidate_id = None;
        self.candidate_hash = None;
        self.candidate_bytes = None;
        self.candidate_dimensions = None;
        self.savings_bytes = None;
        self.savings_percent = None;
        self
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
