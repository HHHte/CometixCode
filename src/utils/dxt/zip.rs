//! Maps to: CC `utils/dxt/zip.ts`. fflate's raw DEFLATE backend is represented
//! by flate2; source path/size/ratio validation and central-directory mode
//! parsing stay here. ZIP64 fields and fflate Latin-1/UTF-8 names are decoded.
//! Malformed/truncated archive diagnostics remain native decoder wording.
use std::collections::HashMap;

/// Maps to: CC `utils/dxt/zip.ts:18-23#ZipValidationState`.
pub struct ZipValidationState {
    pub file_count: f64,
    pub total_uncompressed_size: f64,
    pub compressed_size: f64,
    pub errors: Vec<String>,
}
/// Maps to: CC `utils/dxt/zip.ts:28-31#ZipFileMetadata`.
pub struct ZipFileMetadata<'a> {
    pub name: &'a str,
    pub original_size: Option<f64>,
}
/// Maps to: CC `utils/dxt/zip.ts:36-39#FileValidationResult`.
pub struct FileValidationResult {
    pub is_valid: bool,
    pub error: Option<String>,
}
/// Native carrier for fflate's numeric error codes (unzipFile dependency).
#[derive(Debug)]
pub struct ZipParseError(pub String);
impl std::fmt::Display for ZipParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ZipParseError {}
/// Maps to: CC `utils/dxt/zip.ts:44-58#isPathSafe`.
pub fn is_path_safe(path: &str) -> bool {
    !crate::utils::path::contains_path_traversal(path) && !std::path::Path::new(path).is_absolute()
}
/// Maps to: CC `utils/dxt/zip.ts:63-102#validateZipFile`.
pub fn validate_zip_file(
    file: ZipFileMetadata<'_>,
    state: &mut ZipValidationState,
) -> FileValidationResult {
    state.file_count += 1.0;
    let mut error = None;
    if state.file_count > 100000.0 {
        error = Some(format!(
            "Archive contains too many files: {} (max: 100000)",
            ryu_js::Buffer::new().format(state.file_count)
        ));
    }
    if !is_path_safe(file.name) {
        error = Some(format!(
            "Unsafe file path detected: \"{}\". Path traversal or absolute paths are not allowed.",
            file.name
        ));
    }
    let size = file
        .original_size
        .filter(|n| !n.is_nan() && *n != 0.0)
        .unwrap_or(0.0);
    if size > 512.0 * 1024.0 * 1024.0 {
        error = Some(format!(
            "File \"{}\" is too large: {}MB (max: 512MB)",
            file.name,
            ryu_js::Buffer::new().format((size / 1024.0 / 1024.0).round())
        ));
    }
    state.total_uncompressed_size += size;
    if state.total_uncompressed_size > 1024.0 * 1024.0 * 1024.0 {
        error = Some(format!(
            "Archive total size is too large: {}MB (max: 1024MB)",
            ryu_js::Buffer::new().format((state.total_uncompressed_size / 1024.0 / 1024.0).round())
        ));
    }
    let ratio = state.total_uncompressed_size / state.compressed_size;
    if ratio > 50.0 {
        let ratio = if ratio.is_infinite() {
            "Infinity".into()
        } else {
            ryu_js::Buffer::new().format_to_fixed(ratio, 1).to_owned()
        };
        error = Some(format!(
            "Suspicious compression ratio detected: {ratio}:1 (max: 50:1). This may be a zip bomb."
        ));
    }
    FileValidationResult {
        is_valid: error.is_none(),
        error,
    }
}
/// Maps to: CC `utils/dxt/zip.ts:113-141#unzipFile`.
/// Native ZIP transport preserves central-directory enumeration/duplicate-key
/// replacement order, raw bytes and validation-before-decompression ordering.
pub async fn unzip_file(data: &[u8]) -> anyhow::Result<indexmap::IndexMap<String, Vec<u8>>> {
    use std::io::Read;
    let mut result = indexmap::IndexMap::new();
    let mut state = ZipValidationState {
        file_count: 0.0,
        total_uncompressed_size: 0.0,
        compressed_size: data.len() as f64,
        errors: vec![],
    };
    let read16 = |i: usize| -> anyhow::Result<u16> {
        Ok(u16::from_le_bytes(
            data.get(i..i + 2)
                .ok_or_else(|| ZipParseError("invalid zip data".into()))?
                .try_into()
                .unwrap(),
        ))
    };
    let read32 = |i: usize| -> anyhow::Result<u32> {
        Ok(u32::from_le_bytes(
            data.get(i..i + 4)
                .ok_or_else(|| ZipParseError("invalid zip data".into()))?
                .try_into()
                .unwrap(),
        ))
    };
    let eocd = (0..=data.len().saturating_sub(22))
        .rev()
        .take(65536)
        .find(|&i| data.get(i..i + 4) == Some(&[0x50, 0x4b, 0x05, 0x06]))
        .ok_or_else(|| ZipParseError("invalid zip data".into()))?;
    let mut count = read16(eocd + 8)? as u32;
    let mut off = read32(eocd + 16)? as usize;
    let mut zip64 = off == u32::MAX as usize || count == u16::MAX as u32;
    if zip64 {
        let zoff = read32(
            eocd.checked_sub(12)
                .ok_or_else(|| ZipParseError("invalid zip data".into()))?,
        )? as usize;
        zip64 = read32(zoff)? == 0x06064b50;
        if zip64 {
            count = read32(zoff + 32)?;
            off = read32(zoff + 48)? as usize;
        }
    }
    for _ in 0..count {
        let method = read16(off + 10)?;
        let mut compressed = read32(off + 20)? as usize;
        let mut size = read32(off + 24)? as usize;
        let name_len = read16(off + 28)? as usize;
        let extra_len = read16(off + 30)? as usize;
        let comment_len = read16(off + 32)? as usize;
        let mut local = read32(off + 42)? as usize;
        let raw_name = data
            .get(off + 46..off + 46 + name_len)
            .ok_or_else(|| ZipParseError("invalid zip data".into()))?;
        let name = if read16(off + 8)? & 2048 != 0 {
            String::from_utf8_lossy(raw_name).into_owned()
        } else {
            raw_name.iter().map(|&b| char::from(b)).collect()
        };
        if zip64 && compressed == u32::MAX as usize {
            let mut extra = off + 46 + name_len;
            while read16(extra)? != 1 {
                extra += 4 + read16(extra + 2)? as usize;
            }
            let read64 = |i: usize| -> anyhow::Result<usize> {
                Ok(u64::from_le_bytes(
                    data.get(i..i + 8)
                        .ok_or_else(|| ZipParseError("invalid zip data".into()))?
                        .try_into()
                        .unwrap(),
                ) as usize)
            };
            compressed = read64(extra + 12)?;
            size = read64(extra + 4)?;
            local = read64(extra + 20)?;
        }
        let validation = validate_zip_file(
            ZipFileMetadata {
                name: &name,
                original_size: Some(size as f64),
            },
            &mut state,
        );
        if let Some(error) = validation.error {
            anyhow::bail!(error);
        }
        let begin = local + 30 + read16(local + 26)? as usize + read16(local + 28)? as usize;
        let bytes = data
            .get(begin..begin + compressed)
            .ok_or_else(|| ZipParseError("invalid zip data".into()))?;
        let output = match method {
            0 => bytes.to_vec(),
            8 => {
                let mut decoded = Vec::new();
                let mut decoder = flate2::read::DeflateDecoder::new(bytes);
                (&mut decoder)
                    .take(size as u64)
                    .read_to_end(&mut decoded)
                    .map_err(|e| ZipParseError(e.to_string()))?;
                // fflate receives a fixed Uint8Array(originalSize): excess
                // decoded bytes do not grow it, but decoding still completes.
                std::io::copy(&mut decoder, &mut std::io::sink())
                    .map_err(|e| ZipParseError(e.to_string()))?;
                decoded
            }
            _ => return Err(ZipParseError(format!("unknown compression type {method}")).into()),
        };
        result.insert(name, output);
        off += 46 + name_len + extra_len + comment_len;
    }
    crate::utils::debug::log_for_debugging(&format!(
        "Zip extraction completed: {} files, {}KB uncompressed",
        ryu_js::Buffer::new().format(state.file_count),
        ryu_js::Buffer::new().format((state.total_uncompressed_size / 1024.0).round())
    ));
    Ok(result)
}
/// Maps to: CC `utils/dxt/zip.ts:160-203#parseZipModes`.
pub fn parse_zip_modes(data: &[u8]) -> HashMap<String, u32> {
    let mut modes = HashMap::new();
    if data.len() < 22 {
        return modes;
    }
    let eocd = (data.len().saturating_sub(22 + 65535)..=data.len() - 22)
        .rev()
        .find(|&i| data[i..i + 4] == [0x50, 0x4b, 0x05, 0x06]);
    let Some(eocd) = eocd else { return modes };
    let count = u16::from_le_bytes(data[eocd + 10..eocd + 12].try_into().unwrap());
    let mut off = u32::from_le_bytes(data[eocd + 16..eocd + 20].try_into().unwrap()) as usize;
    for _ in 0..count {
        if off + 46 > data.len() || data[off..off + 4] != [0x50, 0x4b, 0x01, 0x02] {
            break;
        }
        let made = u16::from_le_bytes(data[off + 4..off + 6].try_into().unwrap());
        let name_len = u16::from_le_bytes(data[off + 28..off + 30].try_into().unwrap()) as usize;
        let extra_len = u16::from_le_bytes(data[off + 30..off + 32].try_into().unwrap()) as usize;
        let comment_len = u16::from_le_bytes(data[off + 32..off + 34].try_into().unwrap()) as usize;
        let attr = u32::from_le_bytes(data[off + 38..off + 42].try_into().unwrap());
        let name = String::from_utf8_lossy(&data[off + 46..(off + 46 + name_len).min(data.len())])
            .into_owned();
        if made >> 8 == 3 {
            let mode = (attr >> 16) & 0xffff;
            if mode != 0 {
                modes.insert(name, mode);
            }
        }
        off += 46 + name_len + extra_len + comment_len;
    }
    modes
}
/// Maps to: CC `utils/dxt/zip.ts:209-226#readAndUnzipFile`.
pub async fn read_and_unzip_file(
    path: &std::path::Path,
) -> anyhow::Result<indexmap::IndexMap<String, Vec<u8>>> {
    let result: anyhow::Result<_> = async {
        let bytes = tokio::fs::read(path).await?;
        unzip_file(&bytes).await
    }
    .await;
    result.map_err(|e| {
        if crate::utils::errors::is_enoent(&e) {
            e
        } else {
            anyhow::anyhow!("Failed to read or unzip file: {e}")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zip_validation_order_and_mode_corruption_match_original() {
        for (path, safe) in [
            ("../a", false),
            ("a/../b", false),
            ("/abs", false),
            ("a\\..\\b", false),
            ("a/./b", true),
            ("", true),
        ] {
            assert_eq!(is_path_safe(path), safe);
        }
        let mut state = ZipValidationState {
            file_count: 100000.0,
            total_uncompressed_size: 0.0,
            compressed_size: 1.0,
            errors: vec![],
        };
        let result = validate_zip_file(
            ZipFileMetadata {
                name: "../unsafe",
                original_size: Some(600.0 * 1024.0 * 1024.0),
            },
            &mut state,
        );
        assert!(
            result
                .error
                .unwrap()
                .starts_with("Suspicious compression ratio detected:")
        );
        assert_eq!(state.file_count, 100001.0);
        assert!(parse_zip_modes(&[]).is_empty());
        assert!(parse_zip_modes(&[0; 21]).is_empty());
    }
    #[tokio::test]
    async fn malformed_zip_classifies_as_numeric_fflate_error() {
        let error = unzip_file(b"not zip").await.unwrap_err();
        assert!(error.is::<ZipParseError>());
    }
}
