use std::collections::BTreeSet;
use std::io::Cursor;

use oxipng::{Options, StripChunks};

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

pub(super) struct OptimizedPng {
    pub(super) bytes: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) present_metadata_classes: Vec<&'static str>,
}

pub(super) fn optimize_losslessly(input: &[u8]) -> Result<OptimizedPng, PngOptimizationError> {
    if !input.starts_with(PNG_SIGNATURE) {
        return Err(PngOptimizationError::NotPng);
    }

    let original = decode(input)?;
    let original_chunks = chunks_except_image_data(input)?;
    let mut options = Options::from_preset(3);
    options.interlace = None;
    options.optimize_alpha = false;
    options.bit_depth_reduction = false;
    options.color_type_reduction = false;
    options.palette_reduction = false;
    options.grayscale_reduction = false;
    options.scale_16 = false;
    options.strip = StripChunks::None;
    let candidate = oxipng::optimize_from_memory(input, &options)
        .map_err(|error| PngOptimizationError::Codec(error.to_string()))?;

    let optimized = decode(&candidate)?;
    if original != optimized {
        return Err(PngOptimizationError::Preservation(
            "decoded pixels, transparency, or image properties changed",
        ));
    }
    if original_chunks != chunks_except_image_data(&candidate)? {
        return Err(PngOptimizationError::Preservation(
            "a non-image PNG chunk changed",
        ));
    }

    Ok(OptimizedPng {
        bytes: candidate,
        width: original.width,
        height: original.height,
        present_metadata_classes: metadata_classes(&original_chunks),
    })
}

#[derive(Debug, PartialEq, Eq)]
struct DecodedPng {
    width: u32,
    height: u32,
    color_type: png::ColorType,
    bit_depth: png::BitDepth,
    pixels: Vec<u8>,
}

fn decode(bytes: &[u8]) -> Result<DecodedPng, PngOptimizationError> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|error| PngOptimizationError::Codec(error.to_string()))?;
    let mut pixels = vec![
        0;
        reader.output_buffer_size().ok_or_else(|| {
            PngOptimizationError::Codec("decoded image exceeds addressable memory".into())
        })?
    ];
    let output = reader
        .next_frame(&mut pixels)
        .map_err(|error| PngOptimizationError::Codec(error.to_string()))?;
    pixels.truncate(output.buffer_size());
    Ok(DecodedPng {
        width: output.width,
        height: output.height,
        color_type: output.color_type,
        bit_depth: output.bit_depth,
        pixels,
    })
}

fn chunks_except_image_data(bytes: &[u8]) -> Result<Vec<Chunk>, PngOptimizationError> {
    let mut offset = PNG_SIGNATURE.len();
    let mut chunks = Vec::new();
    while offset < bytes.len() {
        let header_end = offset
            .checked_add(8)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| PngOptimizationError::Codec("truncated chunk header".into()))?;
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("length was checked"),
        ) as usize;
        let chunk_end = header_end
            .checked_add(length)
            .and_then(|end| end.checked_add(4))
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| PngOptimizationError::Codec("truncated chunk data".into()))?;
        let name: [u8; 4] = bytes[offset + 4..header_end]
            .try_into()
            .expect("length was checked");
        if name != *b"IDAT" {
            chunks.push(Chunk {
                name,
                data: bytes[header_end..header_end + length].to_vec(),
            });
        }
        offset = chunk_end;
        if name == *b"IEND" {
            return Ok(chunks);
        }
    }
    Err(PngOptimizationError::Codec("PNG has no IEND chunk".into()))
}

#[derive(Debug, PartialEq, Eq)]
struct Chunk {
    name: [u8; 4],
    data: Vec<u8>,
}

fn metadata_classes(chunks: &[Chunk]) -> Vec<&'static str> {
    let mut classes = BTreeSet::new();
    for chunk in chunks {
        let class = match &chunk.name {
            b"cHRM" | b"gAMA" | b"iCCP" | b"sRGB" | b"cICP" | b"mDCV" | b"cLLI" => {
                Some("color_appearance")
            }
            b"eXIf" => Some("exif"),
            b"pHYs" => Some("physical_dimensions"),
            b"tEXt" | b"zTXt" | b"iTXt" => Some("text"),
            name if name[0] & 0x20 != 0 => Some("unknown_ancillary"),
            _ => None,
        };
        if let Some(class) = class {
            classes.insert(class);
        }
    }
    classes.into_iter().collect()
}

#[derive(Debug)]
pub(super) enum PngOptimizationError {
    NotPng,
    Codec(String),
    Preservation(&'static str),
}

impl std::fmt::Display for PngOptimizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPng => formatter.write_str("input bytes do not have a PNG signature"),
            Self::Codec(error) => formatter.write_str(error),
            Self::Preservation(detail) => {
                write!(formatter, "preservation validation failed: {detail}")
            }
        }
    }
}

impl std::error::Error for PngOptimizationError {}
