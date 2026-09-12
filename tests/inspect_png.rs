use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

#[test]
fn inspect_valid_png_by_content_retains_a_complete_immutable_candidate() {
    let fixture = fixture_png();
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("misleading-extension.dat");
    let candidate_root = temp.path().join("caller-selected-candidates");
    fs::write(&source, &fixture).unwrap();
    let source_before = fs::read(&source).unwrap();

    let output = inspect(&source, &candidate_root);

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert!(output.stderr.is_empty(), "unexpected diagnostic output");
    let report = one_json_document(&output.stdout);
    let session_id = report["session_id"].as_str().unwrap();
    uuid::Uuid::parse_str(session_id).unwrap();
    assert_eq!(
        report["summary"],
        json!({"candidate": 1, "unchanged": 0, "skipped": 0, "failed": 0})
    );
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["command"], "inspect");
    assert_eq!(report["items"].as_array().unwrap().len(), 1);

    let item = &report["items"][0];
    let canonical_source = fs::canonicalize(&source).unwrap();
    assert_eq!(item["outcome"], "candidate");
    assert_eq!(item["reason_code"], "smaller_lossless_candidate");
    assert_eq!(item["format"], "png");
    assert_eq!(item["source"], canonical_source.to_str().unwrap());
    assert_eq!(item["destination"], canonical_source.to_str().unwrap());
    assert_eq!(item["source_hash"], sha256(&fixture));
    assert_eq!(item["destination_hash"], sha256(&fixture));
    assert_eq!(item["original_bytes"], fixture.len() as u64);
    assert_eq!(
        item["original_dimensions"],
        json!({"width": 64, "height": 64})
    );
    assert_eq!(
        item["candidate_dimensions"],
        json!({"width": 64, "height": 64})
    );
    assert_eq!(
        item["resolved_policy"],
        json!({"preset": "lossless", "policy_id": "png-lossless-v1"})
    );
    assert_eq!(item["operations"], json!(["lossless_compression"]));
    assert_eq!(
        item["metadata_policy"],
        json!({
            "mode": "preserve",
            "supported_classes": [
                "color_appearance",
                "exif",
                "physical_dimensions",
                "text",
                "unknown_ancillary"
            ],
            "present_classes": [
                "color_appearance",
                "exif",
                "physical_dimensions",
                "text",
                "unknown_ancillary"
            ]
        })
    );

    let candidate_id = item["candidate_id"].as_str().unwrap();
    uuid::Uuid::parse_str(candidate_id).unwrap();
    let candidate_path = candidate_root
        .join(session_id)
        .join(format!("{candidate_id}.png"));
    let candidate = fs::read(&candidate_path).unwrap();
    assert!(candidate.len() < fixture.len());
    assert_eq!(item["candidate_bytes"], candidate.len() as u64);
    assert_eq!(item["candidate_hash"], sha256(&candidate));
    assert_eq!(
        item["savings_bytes"],
        (fixture.len() - candidate.len()) as u64
    );
    assert!(item["savings_percent"].as_f64().unwrap() > 0.0);
    assert_eq!(decoded(&candidate), decoded(&fixture));
    assert_eq!(non_image_chunks(&candidate), non_image_chunks(&fixture));
    assert_eq!(fs::read(&source).unwrap(), source_before);
    assert_read_only(&candidate_path);

    let manifest_path = candidate_root.join(session_id).join("manifest.json");
    let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["manifest_version"], 1);
    assert_eq!(manifest["session_id"], session_id);
    assert_eq!(manifest["items"], report["items"]);
    assert_read_only(&manifest_path);
}

#[test]
fn inspect_without_candidate_directory_uses_the_per_user_cache() {
    let temp = TempDir::new().unwrap();
    let isolated_home = temp.path().join("home");
    let isolated_xdg_cache = temp.path().join("xdg-cache");
    let source = temp.path().join("source.png");
    fs::create_dir(&isolated_home).unwrap();
    fs::write(&source, fixture_png()).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_image-optimizer"));
    command.arg("inspect").arg(&source);
    if cfg!(target_os = "macos") {
        command
            .env("HOME", &isolated_home)
            .env("XDG_CACHE_HOME", &isolated_xdg_cache);
    } else {
        command
            .env_remove("HOME")
            .env("XDG_CACHE_HOME", &isolated_xdg_cache);
    }
    let output = command.output().unwrap();

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert!(output.stderr.is_empty(), "unexpected diagnostic output");
    let report = one_json_document(&output.stdout);
    let session_id = report["session_id"].as_str().unwrap();
    let cache_root = if cfg!(target_os = "macos") {
        isolated_home.join("Library/Caches/image-optimizer")
    } else if cfg!(target_os = "linux") {
        isolated_xdg_cache.join("image-optimizer")
    } else {
        panic!("default candidate storage is only specified for macOS and Linux");
    };
    let session_dir = cache_root.join(session_id);
    assert!(session_dir.join("manifest.json").is_file());
    let candidate_id = report["items"][0]["candidate_id"].as_str().unwrap();
    assert!(session_dir.join(format!("{candidate_id}.png")).is_file());
}

#[cfg(target_os = "linux")]
#[test]
fn inspect_without_xdg_cache_uses_the_linux_home_cache() {
    let temp = TempDir::new().unwrap();
    let isolated_home = temp.path().join("home");
    let source = temp.path().join("source.png");
    fs::create_dir(&isolated_home).unwrap();
    fs::write(&source, fixture_png()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_image-optimizer"))
        .arg("inspect")
        .arg(&source)
        .env("HOME", &isolated_home)
        .env_remove("XDG_CACHE_HOME")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert!(output.stderr.is_empty(), "unexpected diagnostic output");
    let report = one_json_document(&output.stdout);
    let session_id = report["session_id"].as_str().unwrap();
    let session_dir = isolated_home
        .join(".cache/image-optimizer")
        .join(session_id);
    assert!(session_dir.join("manifest.json").is_file());
    let candidate_id = report["items"][0]["candidate_id"].as_str().unwrap();
    assert!(session_dir.join(format!("{candidate_id}.png")).is_file());
}

#[test]
fn inspect_valid_png_without_strict_savings_retains_no_candidate_image() {
    let temp = TempDir::new().unwrap();
    let initial_source = temp.path().join("initial.png");
    let initial_store = temp.path().join("initial-store");
    fs::write(&initial_source, fixture_png()).unwrap();
    let initial_report = one_json_document(&inspect(&initial_source, &initial_store).stdout);
    let optimized = fs::read(
        initial_store
            .join(initial_report["session_id"].as_str().unwrap())
            .join(format!(
                "{}.png",
                initial_report["items"][0]["candidate_id"].as_str().unwrap()
            )),
    )
    .unwrap();

    let source = temp.path().join("already-optimized.unknown");
    let candidate_root = temp.path().join("unchanged-store");
    fs::write(&source, &optimized).unwrap();
    let source_before = fs::read(&source).unwrap();

    let output = inspect(&source, &candidate_root);

    assert!(output.status.success(), "stderr: {}", text(&output.stderr));
    assert!(output.stderr.is_empty());
    let report = one_json_document(&output.stdout);
    assert_eq!(
        report["summary"],
        json!({"candidate": 0, "unchanged": 1, "skipped": 0, "failed": 0})
    );
    let item = &report["items"][0];
    assert_eq!(item["outcome"], "unchanged");
    assert_eq!(item["reason_code"], "not_strictly_smaller");
    assert_eq!(item["format"], "png");
    let canonical_source = fs::canonicalize(&source).unwrap();
    assert_eq!(item["source"], canonical_source.to_str().unwrap());
    assert_eq!(item["destination"], canonical_source.to_str().unwrap());
    assert_eq!(item["source_hash"], sha256(&optimized));
    assert_eq!(item["destination_hash"], sha256(&optimized));
    assert_eq!(item["original_bytes"], optimized.len() as u64);
    assert_eq!(item["candidate_bytes"], optimized.len() as u64);
    assert_eq!(item["original_dimensions"], item["candidate_dimensions"]);
    assert_eq!(
        item["resolved_policy"],
        json!({"preset": "lossless", "policy_id": "png-lossless-v1"})
    );
    assert_eq!(item["operations"], json!(["lossless_compression"]));
    assert_eq!(item["metadata_policy"]["mode"], "preserve");
    assert!(item.get("candidate_id").is_none());
    assert!(item.get("candidate_hash").is_none());
    assert!(item.get("savings_bytes").is_none());
    assert!(item.get("savings_percent").is_none());
    assert_eq!(fs::read(&source).unwrap(), source_before);

    let session_dir = candidate_root.join(report["session_id"].as_str().unwrap());
    let image_files: Vec<PathBuf> = fs::read_dir(&session_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    assert!(
        image_files.is_empty(),
        "unexpected retained images: {image_files:?}"
    );
    let manifest: Value =
        serde_json::from_slice(&fs::read(session_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["items"], report["items"]);
}

fn inspect(source: &Path, candidate_root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_image-optimizer"))
        .arg("inspect")
        .arg(source)
        .arg("--candidate-dir")
        .arg(candidate_root)
        .output()
        .unwrap()
}

fn one_json_document(stdout: &[u8]) -> Value {
    let mut stream = serde_json::Deserializer::from_slice(stdout).into_iter::<Value>();
    let document = stream.next().expect("one JSON document").unwrap();
    assert!(
        stream.next().is_none(),
        "stdout contained another JSON document"
    );
    document
}

fn fixture_png() -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, 64, 64);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::NoCompression);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45_455));
    encoder.set_pixel_dims(Some(png::PixelDimensions {
        xppu: 3_780,
        yppu: 3_780,
        unit: png::Unit::Meter,
    }));
    encoder
        .add_text_chunk("Description".into(), "lossless metadata fixture".into())
        .unwrap();
    let mut writer = encoder.write_header().unwrap();
    // Minimal little-endian TIFF payload carrying an orientation tag with value 6.
    writer
        .write_chunk(
            png::chunk::eXIf,
            &[
                b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0,
                0, 0, 0,
            ],
        )
        .unwrap();
    writer
        .write_chunk(png::chunk::ChunkType(*b"vpAg"), b"private fixture metadata")
        .unwrap();
    let pixels: Vec<u8> = (0..64 * 64)
        .flat_map(|index| {
            let x = (index % 64) as u8;
            let y = (index / 64) as u8;
            [
                x.wrapping_mul(4),
                y.wrapping_mul(4),
                x ^ y,
                (x + y) % 5 * 51,
            ]
        })
        .collect();
    writer.write_image_data(&pixels).unwrap();
    drop(writer);
    bytes
}

fn decoded(bytes: &[u8]) -> (u32, u32, png::ColorType, png::BitDepth, Vec<u8>) {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let output = reader.next_frame(&mut pixels).unwrap();
    pixels.truncate(output.buffer_size());
    (
        output.width,
        output.height,
        output.color_type,
        output.bit_depth,
        pixels,
    )
}

fn non_image_chunks(bytes: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
    let mut offset = 8;
    let mut chunks = Vec::new();
    while offset < bytes.len() {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let name: [u8; 4] = bytes[offset + 4..offset + 8].try_into().unwrap();
        let data = bytes[offset + 8..offset + 8 + length].to_vec();
        offset += 12 + length;
        if name != *b"IDAT" {
            chunks.push((name, data));
        }
        if name == *b"IEND" {
            break;
        }
    }
    chunks
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(unix)]
fn assert_read_only(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o222, 0);
}

#[cfg(not(unix))]
fn assert_read_only(path: &Path) {
    assert!(fs::metadata(path).unwrap().permissions().readonly());
}
