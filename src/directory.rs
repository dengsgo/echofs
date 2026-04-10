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

/// Resolve a path for write operations (PUT, MKCOL) where the target may not exist yet.
/// Validates: hidden component check, depth check, parent directory exists and is within root.
/// Returns the full (non-canonicalized) target path. The parent is canonicalized and verified.
pub async fn safe_resolve_parent(root: &Path, rel_path: &str, show_hidden: bool, max_depth: i32) -> Result<PathBuf, AppError> {
    let rel_path_owned = rel_path.trim_start_matches('/').to_string();

    if rel_path_owned.is_empty() {
        return Err(AppError::BadRequest("Cannot write to root".into()));
    }

    // Block access to hidden files/directories (any path component starting with '.')
    if !show_hidden && has_hidden_component(&rel_path_owned) {
        return Err(AppError::Forbidden("Access to hidden files is denied".into()));
    }

    let root = root.to_path_buf();
    let rel = rel_path_owned.clone();
    tokio::task::spawn_blocking(move || {
        let target = root.join(&rel);

        // Validate parent directory exists and is within root
        let parent = target.parent().ok_or_else(|| {
            AppError::BadRequest("Invalid path".into())
        })?;

        let canonical_parent = std::fs::canonicalize(parent).map_err(|_| {
            AppError::Conflict("Parent directory does not exist".into())
        })?;

        if !canonical_parent.starts_with(&root) {
            return Err(AppError::Forbidden("Path traversal denied".into()));
        }

        // Check depth: for files the depth is number of segments,
        // for directories (MKCOL) same check applies
        if max_depth >= 0 {
            let depth = path_depth(&rel);
            if depth > (max_depth as u32) + 1 {
                return Err(AppError::Forbidden("Maximum directory depth exceeded".into()));
            }
        }

        Ok(target)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

pub async fn safe_resolve(root: &Path, rel_path: &str, show_hidden: bool, max_depth: i32) -> Result<PathBuf, AppError> {
    let rel_path_owned = rel_path.trim_start_matches('/').to_string();

    // Block access to hidden files/directories (any path component starting with '.')
    if !show_hidden && !rel_path_owned.is_empty() && has_hidden_component(&rel_path_owned) {
        return Err(AppError::Forbidden("Access to hidden files is denied".into()));
    }

    let root = root.to_path_buf();
    let rel = rel_path_owned.clone();
    tokio::task::spawn_blocking(move || {
        let candidate = if rel.is_empty() {
            root.clone()
        } else {
            root.join(&rel)
        };

        let canonical = std::fs::canonicalize(&candidate).map_err(|_| {
            AppError::NotFound(format!("Path not found: {}", rel))
        })?;

        if !canonical.starts_with(&root) {
            return Err(AppError::Forbidden("Path traversal denied".into()));
        }

        // Block access beyond maximum directory depth.
        // For directories: the directory's own depth must be within limit.
        // For files: the file's parent directory depth must be within limit,
        // so a file at depth N is allowed if N-1 <= max_depth (i.e. N <= max_depth+1).
        if max_depth >= 0 {
            let depth = path_depth(&rel);
            if canonical.is_dir() && depth > max_depth as u32 {
                return Err(AppError::Forbidden("Maximum directory depth exceeded".into()));
            }
            if canonical.is_file() && depth > (max_depth as u32) + 1 {
                return Err(AppError::Forbidden("Maximum directory depth exceeded".into()));
            }
        }

        Ok(canonical)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

/// Check if any component in the path starts with '.'
fn has_hidden_component(rel_path: &str) -> bool {
    rel_path.split('/').any(|component| {
        component.starts_with('.')
    })
}

/// Calculate the depth of a relative path (number of non-empty segments).
/// "" → 0, "a" → 1, "a/b" → 2, "a/b/c" → 3
fn path_depth(rel_path: &str) -> u32 {
    if rel_path.is_empty() {
        return 0;
    }
    rel_path.split('/').filter(|s| !s.is_empty()).count() as u32
}

pub async fn list_directory(root: &Path, rel_path: &str, show_hidden: bool, max_depth: i32) -> Result<DirListing, AppError> {
    let full_path = safe_resolve(root, rel_path, show_hidden, max_depth).await?;

    if !full_path.is_dir() {
        return Err(AppError::BadRequest("Not a directory".into()));
    }

    let rel_path_owned = rel_path.to_string();
    tokio::task::spawn_blocking(move || {
        list_directory_sync(&full_path, &rel_path_owned, show_hidden, max_depth)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
}

fn list_directory_sync(full_path: &Path, rel_path: &str, show_hidden: bool, max_depth: i32) -> Result<DirListing, AppError> {

    let current_depth = path_depth(rel_path.trim_start_matches('/'));
    let mut entries = Vec::new();
    let rd = std::fs::read_dir(full_path).map_err(AppError::from)?;

    for entry in rd {
        let entry = entry.map_err(AppError::from)?;
        let metadata = entry.metadata().map_err(AppError::from)?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let is_dir = metadata.is_dir();

        // Skip subdirectories when at maximum depth
        if max_depth >= 0 && is_dir && current_depth >= max_depth as u32 {
            continue;
        }
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

    // ─── Helper: create temp root ───

    fn tmp_root() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        (tmp, root)
    }

    // ─── format_size ───

    #[test]
    fn format_size_values() {
        let cases: Vec<(u64, &str)> = vec![
            (0,                              "-"),
            (100,                            "100 B"),
            (1024,                           "1.0 KB"),
            (1024 * 1024,                    "1.0 MB"),
            (1024 * 1024 * 1024,             "1.0 GB"),
            (1024u64 * 1024 * 1024 * 1024,   "1.0 TB"),
        ];
        for (bytes, expected) in cases {
            assert_eq!(format_size(bytes), expected, "format_size({})", bytes);
        }
    }

    // ─── safe_resolve ───

    #[tokio::test]
    async fn safe_resolve_basic() {
        let (_tmp, root) = tmp_root();
        fs::write(root.join("hello.txt"), "hi").unwrap();
        fs::write(root.join("file.txt"), "data").unwrap();
        fs::create_dir(root.join("subdir")).unwrap();

        // Root resolves to itself
        assert_eq!(safe_resolve(&root, "", false, -1).await.unwrap(), root);
        // File resolves correctly
        assert_eq!(safe_resolve(&root, "hello.txt", false, -1).await.unwrap(), root.join("hello.txt"));
        // Leading slash is stripped
        assert_eq!(safe_resolve(&root, "/file.txt", false, -1).await.unwrap(), root.join("file.txt"));
        // Subdirectory resolves
        assert_eq!(safe_resolve(&root, "subdir", false, -1).await.unwrap(), root.join("subdir"));
        // Nonexistent path errors
        assert!(safe_resolve(&root, "nonexistent.txt", false, -1).await.is_err());
    }

    #[tokio::test]
    async fn safe_resolve_hidden_paths_denied() {
        let (_tmp, root) = tmp_root();
        fs::write(root.join(".secret"), "data").unwrap();
        fs::create_dir_all(root.join("public/.hidden")).unwrap();
        fs::write(root.join("public/.hidden/secret.txt"), "data").unwrap();

        assert!(safe_resolve(&root, "/.secret", false, -1).await.is_err());
        assert!(safe_resolve(&root, "public/.hidden/secret.txt", false, -1).await.is_err());
    }

    // ─── list_directory ───

    #[tokio::test]
    async fn list_directory_empty() {
        let (_tmp, root) = tmp_root();
        let listing = list_directory(&root, "", false, -1).await.unwrap();
        assert!(listing.entries.is_empty());
        assert_eq!(listing.path, "/");
    }

    #[tokio::test]
    async fn list_directory_files_and_dirs_sorted() {
        let (_tmp, root) = tmp_root();
        fs::write(root.join("beta.txt"), "b").unwrap();
        fs::write(root.join("alpha.txt"), "a").unwrap();
        fs::create_dir(root.join("zdir")).unwrap();

        let listing = list_directory(&root, "", false, -1).await.unwrap();
        assert!(listing.entries[0].is_dir);
        assert_eq!(listing.entries[0].name, "zdir");
        assert_eq!(listing.entries[1].name, "alpha.txt");
        assert_eq!(listing.entries[2].name, "beta.txt");
    }

    #[tokio::test]
    async fn list_directory_file_not_dir_errors() {
        let (_tmp, root) = tmp_root();
        fs::write(root.join("file.txt"), "data").unwrap();
        assert!(list_directory(&root, "file.txt", false, -1).await.is_err());
    }

    #[tokio::test]
    async fn list_directory_href_percent_encoding() {
        let (_tmp, root) = tmp_root();
        fs::write(root.join("my file.txt"), "data").unwrap();
        let listing = list_directory(&root, "", false, -1).await.unwrap();
        assert!(listing.entries[0].href.contains("my%20file.txt"));
    }

    // ─── has_hidden_component ───

    #[test]
    fn has_hidden_component_cases() {
        let cases = [
            (".env",               true),
            (".git/config",        true),
            ("foo/.hidden/bar",    true),
            ("foo/bar/baz.txt",    false),
            ("",                   false),
        ];
        for (path, expected) in cases {
            assert_eq!(has_hidden_component(path), expected,
                "has_hidden_component({:?})", path);
        }
    }

    // ─── path_depth ───

    #[test]
    fn path_depth_cases() {
        let cases = [
            ("",                0),
            ("photos",          1),
            ("photos/vacation", 2),
            ("a/b/c",           3),
            ("/photos",         1),  // leading slash ignored
            ("photos/",         1),  // trailing slash ignored
            ("a//b",            2),  // consecutive slashes → empty segments filtered
            ("/",               0),  // only slashes
        ];
        for (path, expected) in cases {
            assert_eq!(path_depth(path), expected, "path_depth({:?})", path);
        }
    }
}
