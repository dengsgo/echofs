use crate::error::AppError;
use crate::mime_utils;
use chrono::{DateTime, Local};
use percent_encoding::{AsciiSet, CONTROLS};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Characters that need percent-encoding in a URL path segment.
const PATH_SEGMENT_ENCODE: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'[')
    .add(b']');

#[derive(Serialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub size_display: String,
    pub created: String,
    pub modified: String,
    pub created_ts: i64,
    pub modified_ts: i64,
    pub icon: String,
    pub href: String,
    pub media_type: String,
}

#[derive(Serialize)]
pub struct Breadcrumb {
    pub name: String,
    pub href: String,
}

#[derive(Serialize)]
pub struct DirListing {
    pub path: String,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub entries: Vec<DirEntry>,
}

pub fn safe_resolve(root: &Path, rel_path: &str) -> Result<PathBuf, AppError> {
    let rel_path = rel_path.trim_start_matches('/');
    let candidate = if rel_path.is_empty() {
        root.to_path_buf()
    } else {
        root.join(rel_path)
    };

    let canonical = std::fs::canonicalize(&candidate).map_err(|_| {
        AppError::NotFound(format!("Path not found: {}", rel_path))
    })?;

    if !canonical.starts_with(root) {
        return Err(AppError::Forbidden("Path traversal denied".into()));
    }

    Ok(canonical)
}

pub fn list_directory(root: &Path, rel_path: &str) -> Result<DirListing, AppError> {
    let full_path = safe_resolve(root, rel_path)?;

    if !full_path.is_dir() {
        return Err(AppError::BadRequest("Not a directory".into()));
    }

    let mut entries = Vec::new();
    let rd = std::fs::read_dir(&full_path).map_err(AppError::from)?;

    for entry in rd {
        let entry = entry.map_err(AppError::from)?;
        let metadata = entry.metadata().map_err(AppError::from)?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };

        let created = metadata
            .created()
            .ok()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_default();

        let created_ts = metadata
            .created()
            .ok()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.timestamp()
            })
            .unwrap_or(0);

        let modified = metadata
            .modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_default();

        let modified_ts = metadata
            .modified()
            .ok()
            .map(|t| {
                let dt: DateTime<Local> = t.into();
                dt.timestamp()
            })
            .unwrap_or(0);

        let icon = mime_utils::icon_for_path(&entry.path(), is_dir).to_string();

        let rel = rel_path.trim_start_matches('/');
        let href = if rel.is_empty() {
            format!("/{}", percent_encoding::utf8_percent_encode(&name, PATH_SEGMENT_ENCODE))
        } else {
            format!("/{}/{}", rel, percent_encoding::utf8_percent_encode(&name, PATH_SEGMENT_ENCODE))
        };

        let mime = mime_utils::detect_mime(&entry.path());
        let media_type = if is_dir {
            "directory".to_string()
        } else if mime_utils::is_video(&mime) {
            "video".to_string()
        } else if mime_utils::is_audio(&mime) {
            "audio".to_string()
        } else if mime_utils::is_image(&mime) {
            "image".to_string()
        } else {
            "other".to_string()
        };

        entries.push(DirEntry {
            name,
            is_dir,
            size,
            size_display: format_size(size),
            created,
            modified,
            created_ts,
            modified_ts,
            icon,
            href,
            media_type,
        });
    }

    // Sort: directories first, then by name
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let breadcrumbs = build_breadcrumbs(rel_path);

    Ok(DirListing {
        path: if rel_path.is_empty() { "/".to_string() } else { format!("/{}", rel_path) },
        breadcrumbs,
        entries,
    })
}

fn build_breadcrumbs(rel_path: &str) -> Vec<Breadcrumb> {
    let mut crumbs = vec![Breadcrumb {
        name: "Home".to_string(),
        href: "/".to_string(),
    }];

    let rel = rel_path.trim_start_matches('/');
    if rel.is_empty() {
        return crumbs;
    }

    let mut accumulated = String::new();
    for part in rel.split('/') {
        if part.is_empty() {
            continue;
        }
        if accumulated.is_empty() {
            accumulated = part.to_string();
        } else {
            accumulated = format!("{}/{}", accumulated, part);
        }
        crumbs.push(Breadcrumb {
            name: part.to_string(),
            href: format!("/{}", accumulated),
        });
    }

    crumbs
}

pub fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "-".to_string();
    }
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn format_size_zero() {
        assert_eq!(format_size(0), "-");
    }

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(100), "100 B");
    }

    #[test]
    fn format_size_kb() {
        assert_eq!(format_size(1024), "1.0 KB");
    }

    #[test]
    fn format_size_mb() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn format_size_gb() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn format_size_tb() {
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 1024), "1.0 TB");
    }

    #[test]
    fn safe_resolve_root_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let resolved = safe_resolve(&root, "").unwrap();
        assert_eq!(resolved, root);
    }

    #[test]
    fn safe_resolve_valid_child() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join("hello.txt"), "hi").unwrap();
        let resolved = safe_resolve(&root, "hello.txt").unwrap();
        assert_eq!(resolved, root.join("hello.txt"));
    }

    #[test]
    fn safe_resolve_strips_leading_slash() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join("file.txt"), "data").unwrap();
        let resolved = safe_resolve(&root, "/file.txt").unwrap();
        assert_eq!(resolved, root.join("file.txt"));
    }

    #[test]
    fn safe_resolve_traversal_denied() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let result = safe_resolve(&root, "../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn safe_resolve_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let result = safe_resolve(&root, "nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn safe_resolve_subdirectory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::create_dir(root.join("subdir")).unwrap();
        let resolved = safe_resolve(&root, "subdir").unwrap();
        assert_eq!(resolved, root.join("subdir"));
    }

    #[test]
    fn list_directory_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let listing = list_directory(&root, "").unwrap();
        assert!(listing.entries.is_empty());
        assert_eq!(listing.path, "/");
    }

    #[test]
    fn list_directory_files_and_dirs_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join("beta.txt"), "b").unwrap();
        fs::write(root.join("alpha.txt"), "a").unwrap();
        fs::create_dir(root.join("zdir")).unwrap();
        let listing = list_directory(&root, "").unwrap();
        // Directory should come first
        assert!(listing.entries[0].is_dir);
        assert_eq!(listing.entries[0].name, "zdir");
        // Then files sorted alphabetically
        assert_eq!(listing.entries[1].name, "alpha.txt");
        assert_eq!(listing.entries[2].name, "beta.txt");
    }

    #[test]
    fn list_directory_hidden_files_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join(".hidden"), "secret").unwrap();
        fs::write(root.join("visible.txt"), "data").unwrap();
        let listing = list_directory(&root, "").unwrap();
        assert_eq!(listing.entries.len(), 1);
        assert_eq!(listing.entries[0].name, "visible.txt");
    }

    #[test]
    fn list_directory_file_not_dir_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join("file.txt"), "data").unwrap();
        let result = list_directory(&root, "file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn list_directory_nested_breadcrumbs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::create_dir_all(root.join("a/b")).unwrap();
        let listing = list_directory(&root, "a/b").unwrap();
        assert_eq!(listing.breadcrumbs.len(), 3); // Home, a, b
        assert_eq!(listing.breadcrumbs[0].name, "Home");
        assert_eq!(listing.breadcrumbs[1].name, "a");
        assert_eq!(listing.breadcrumbs[2].name, "b");
    }

    #[test]
    fn list_directory_href_percent_encoding() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        fs::write(root.join("my file.txt"), "data").unwrap();
        let listing = list_directory(&root, "").unwrap();
        assert!(listing.entries[0].href.contains("my%20file.txt"));
    }
}
