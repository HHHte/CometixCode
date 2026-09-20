//! Binary file extension helpers.
//!
//! Maps to: CC `constants/files.ts` (`BINARY_EXTENSIONS`, `hasBinaryExtension`).

use std::collections::HashSet;
use std::sync::LazyLock;

/// Maps to: CC `BINARY_EXTENSIONS`.
static BINARY_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".webp", ".tiff", ".tif", ".mp4", ".mov",
        ".avi", ".mkv", ".webm", ".wmv", ".flv", ".m4v", ".mpeg", ".mpg", ".mp3", ".wav", ".ogg",
        ".flac", ".aac", ".m4a", ".wma", ".aiff", ".opus", ".zip", ".tar", ".gz", ".bz2", ".7z",
        ".rar", ".xz", ".z", ".tgz", ".iso", ".exe", ".dll", ".so", ".dylib", ".bin", ".o", ".a",
        ".obj", ".lib", ".app", ".msi", ".deb", ".rpm", ".pdf", ".doc", ".docx", ".xls", ".xlsx",
        ".ppt", ".pptx", ".odt", ".ods", ".odp", ".ttf", ".otf", ".woff", ".woff2", ".eot", ".pyc",
        ".pyo", ".class", ".jar", ".war", ".ear", ".node", ".wasm", ".rlib", ".sqlite", ".sqlite3",
        ".db", ".mdb", ".idx", ".psd", ".ai", ".eps", ".sketch", ".fig", ".xd", ".blend", ".3ds",
        ".max", ".swf", ".fla", ".lockb", ".dat", ".data",
    ])
});

/// Maps to: CC `hasBinaryExtension(filePath)`.
pub fn has_binary_extension(file_path: &str) -> bool {
    let Some(dot) = file_path.rfind('.') else {
        return false;
    };
    let ext = file_path[dot..].to_ascii_lowercase();
    BINARY_EXTENSIONS.contains(ext.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_binary_extension_matches_official_set() {
        assert!(has_binary_extension("a.exe"));
        assert!(has_binary_extension("/tmp/photo.PNG"));
        assert!(has_binary_extension("doc.pdf"));
        assert!(!has_binary_extension("src/main.rs"));
        assert!(!has_binary_extension("README"));
    }
}
