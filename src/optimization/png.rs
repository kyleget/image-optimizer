use std::collections::BTreeSet;
use std::io::Cursor;

use oxipng::{Options, StripChunks};

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
// PNG's largest decoded pixel is 16-bit RGBA. Its filtered stream adds one
// filter byte per row, whose worst case is one row per allowed pixel.
const MAX_BYTES_PER_PIXEL: u64 = 8;
const MAX_FILTER_BYTES_PER_PIXEL: u64 = 1;

pub(super) struct OptimizedPng {
    pub(super) bytes: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) present_metadata_classes: Vec<&'static str>,
}

pub(super) fn optimize_losslessly(
    input: &[u8],
    max_decoded_pixels: u64,
) -> Result<OptimizedPng, PngOptimizationError> {
    validate_header_and_pixel_limit(input, max_decoded_pixels)?;

    let original = decode(input, max_decoded_pixels)?;
    let original_chunks = chunks_except_image_data(input)?;
    let options = lossless_options(max_decoded_pixels);
    let candidate = oxipng::optimize_from_memory(input, &options)
        .map_err(|error| PngOptimizationError::Codec(error.to_string()))?;

    let optimized = decode(&candidate, max_decoded_pixels)?;
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

fn lossless_options(max_decoded_pixels: u64) -> Options {
    let mut options = Options::from_preset(3);
    options.interlace = None;
    options.optimize_alpha = false;
    options.bit_depth_reduction = false;
    options.color_type_reduction = false;
    options.palette_reduction = false;
    options.grayscale_reduction = false;
    options.scale_16 = false;
    options.strip = StripChunks::None;
    options.max_decompressed_size = Some(max_decompressed_size(max_decoded_pixels));
    options
}

fn decoded_output_limit(max_decoded_pixels: u64) -> usize {
    usize_limit(max_decoded_pixels.saturating_mul(MAX_BYTES_PER_PIXEL))
}

fn max_decompressed_size(max_decoded_pixels: u64) -> usize {
    usize_limit(max_decoded_pixels.saturating_mul(MAX_BYTES_PER_PIXEL + MAX_FILTER_BYTES_PER_PIXEL))
}

fn decoder_limits(max_decoded_pixels: u64) -> png::Limits {
    png::Limits {
        bytes: max_decompressed_size(max_decoded_pixels),
    }
}

fn usize_limit(limit: u64) -> usize {
    usize::try_from(limit).unwrap_or(usize::MAX)
}

fn validate_header_and_pixel_limit(
    input: &[u8],
    max_decoded_pixels: u64,
) -> Result<(), PngOptimizationError> {
    if !input.starts_with(PNG_SIGNATURE) {
        return Err(PngOptimizationError::NotPng);
    }
    if input.len() < 33 || &input[12..16] != b"IHDR" {
        return Err(PngOptimizationError::Codec(
            "missing or truncated IHDR chunk".to_owned(),
        ));
    }
    let ihdr_length = u32::from_be_bytes(input[8..12].try_into().expect("length was checked"));
    if ihdr_length != 13 {
        return Err(PngOptimizationError::Codec(
            "invalid IHDR chunk length".to_owned(),
        ));
    }
    let width = u32::from_be_bytes(input[16..20].try_into().expect("length was checked"));
    let height = u32::from_be_bytes(input[20..24].try_into().expect("length was checked"));
    if width == 0 || height == 0 {
        return Err(PngOptimizationError::Codec(
            "PNG dimensions must be nonzero".to_owned(),
        ));
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > max_decoded_pixels {
        return Err(PngOptimizationError::PixelLimitExceeded);
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct DecodedPng {
    width: u32,
    height: u32,
    color_type: png::ColorType,
    bit_depth: png::BitDepth,
    pixels: Vec<u8>,
}

fn decode(bytes: &[u8], max_decoded_pixels: u64) -> Result<DecodedPng, PngOptimizationError> {
    let decoder =
        png::Decoder::new_with_limits(Cursor::new(bytes), decoder_limits(max_decoded_pixels));
    let mut reader = decoder
        .read_info()
        .map_err(|error| PngOptimizationError::Codec(error.to_string()))?;
    let output_buffer_size = reader.output_buffer_size().ok_or_else(|| {
        PngOptimizationError::Codec("decoded image exceeds addressable memory".into())
    })?;
    if output_buffer_size > decoded_output_limit(max_decoded_pixels) {
        return Err(PngOptimizationError::PixelLimitExceeded);
    }
    let mut pixels = vec![0; output_buffer_size];
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
    PixelLimitExceeded,
    Preservation(&'static str),
}

impl PngOptimizationError {
    pub(super) fn reason_code(&self) -> &'static str {
        match self {
            Self::PixelLimitExceeded => "decoded_pixel_limit_exceeded",
            Self::Preservation(_) => "preservation_validation_failed",
            Self::NotPng | Self::Codec(_) => "malformed_png",
        }
    }

    pub(super) fn recognized_as_png(&self) -> bool {
        !matches!(self, Self::NotPng)
    }
}

impl std::fmt::Display for PngOptimizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPng => formatter.write_str("input bytes do not have a PNG signature"),
            Self::Codec(error) => formatter.write_str(error),
            Self::PixelLimitExceeded => formatter.write_str("decoded pixel limit exceeded"),
            Self::Preservation(detail) => {
                write!(formatter, "preservation validation failed: {detail}")
            }
        }
    }
}

impl std::error::Error for PngOptimizationError {}

#[cfg(test)]
mod tests {
    use super::{
        MAX_BYTES_PER_PIXEL, decoded_output_limit, decoder_limits, lossless_options,
        max_decompressed_size,
    };

    #[test]
    fn codec_budgets_are_derived_from_the_decoded_pixel_limit() {
        let pixel_limit = 100_000_000;
        let decoded_limit = pixel_limit * MAX_BYTES_PER_PIXEL;
        let scanline_limit = decoded_limit + pixel_limit;

        assert_eq!(decoded_output_limit(pixel_limit), decoded_limit as usize);
        assert_eq!(max_decompressed_size(pixel_limit), scanline_limit as usize);
        assert_eq!(decoder_limits(pixel_limit).bytes, scanline_limit as usize);
        assert_eq!(
            lossless_options(pixel_limit).max_decompressed_size,
            Some(scanline_limit as usize)
        );
    }
}
