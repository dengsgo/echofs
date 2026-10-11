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

    // Fast-path hidden check on the raw request string. This is only a
    // pre-filter: the authoritative check runs on the canonicalized parent
    // below, where the platform's real separator and name-alias semantics
    // apply (on Windows, `\` is a separator and 8.3 short names expand to the
    // real name — a lexical '/'-only check would miss `a\.hidden` and
    // `HIDDEN~1` spellings of hidden components).
    if !show_hidden && has_hidden_component(&rel_path_owned) {
        return Err(AppError::Forbidden("Access to hidden files is denied".into()));
    }

    let root = root.to_path_buf();
    let rel = rel_path_owned.clone();
    tokio::task::spawn_blocking(move || {
        // Canonicalize the root so path-traversal checks compare like-for-like.
        // Without this, a relative or symlinked `--root` would compare a
        // canonicalized parent against a non-canonicalized root, which can
        // misjudge whether the parent lies within the served tree.
        let canonical_root = std::fs::canonicalize(&root).map_err(|_| {
            AppError::Internal("Root directory does not exist".into())
        })?;
        let target = canonical_root.join(&rel);

        // Validate parent directory exists and is within root
        let parent = target.parent().ok_or_else(|| {
            AppError::BadRequest("Invalid path".into())
        })?;

        let canonical_parent = std::fs::canonicalize(parent).map_err(|_| {
            AppError::Conflict("Parent directory does not exist".into())
        })?;

        if !canonical_parent.starts_with(&canonical_root) {
            return Err(AppError::Forbidden("Path traversal denied".into()));
        }

        // The parent's path below the root, in canonical form. The security
        // rulings below operate on real path components, not request strings.
        let parent_rel = canonical_parent.strip_prefix(&canonical_root).map_err(|_| {
            AppError::Internal("Resolved parent escaped the root prefix".into())
        })?;

        // Block hidden components in the (canonical) parent chain, and in the
        // final target name itself. `target.file_name()` parses with platform
        // separator semantics, so on Windows a `dir\.hidden` spelling yields a
        // final name of `.hidden` here.
        if !show_hidden {
            let final_hidden = target
                .file_name()
                .map(|n| n.as_encoded_bytes().starts_with(b"."))
                .unwrap_or(false);
            if path_has_hidden_component(parent_rel) || final_hidden {
                return Err(AppError::Forbidden("Access to hidden files is denied".into()));
            }
        }

        // Check depth: real depth of the parent below the root, plus one for
        // the target itself — for files the depth is number of segments,
        // for directories (MKCOL) same check applies
        if max_depth >= 0 {
            let depth = parent_rel.components().count() as u32 + 1;
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

    // Fast-path hidden check on the raw request string. This is only a
    // pre-filter: the authoritative check runs on the canonicalized path
    // below, where the platform's real separator and name-alias semantics
    // apply (on Windows, `\` is a separator and 8.3 short names expand to the
    // real name — a lexical '/'-only check would miss `a\.hidden` and
    // `HIDDEN~1` spellings of hidden components).
    if !show_hidden && !rel_path_owned.is_empty() && has_hidden_component(&rel_path_owned) {
        return Err(AppError::Forbidden("Access to hidden files is denied".into()));
    }

    let root = root.to_path_buf();
    let rel = rel_path_owned.clone();
    tokio::task::spawn_blocking(move || {
        // Canonicalize the root so path-traversal checks compare like-for-like
        // (a relative or symlinked `--root` otherwise skews the comparison).
        let canonical_root = std::fs::canonicalize(&root).map_err(|_| {
            AppError::Internal("Root directory does not exist".into())
        })?;
        let candidate = if rel.is_empty() {
            canonical_root.clone()
        } else {
            canonical_root.join(&rel)
        };

        let canonical = std::fs::canonicalize(&candidate).map_err(|_| {
            AppError::NotFound(format!("Path not found: {}", rel))
        })?;

        if !canonical.starts_with(&canonical_root) {
            return Err(AppError::Forbidden("Path traversal denied".into()));
        }

        // The resolved path below the root, in canonical form. The security
        // rulings below operate on real path components, not request strings.
        let rel_canonical = canonical.strip_prefix(&canonical_root).map_err(|_| {
            AppError::Internal("Resolved path escaped the root prefix".into())
        })?;

        // Block access to hidden files/directories (any real path component
        // starting with '.').
        if !show_hidden && path_has_hidden_component(rel_canonical) {
            return Err(AppError::Forbidden("Access to hidden files is denied".into()));
        }

        // Block access beyond maximum directory depth.
        // For directories: the directory's own depth must be within limit.
        // For files: the file's parent directory depth must be within limit,
        // so a file at depth N is allowed if N-1 <= max_depth (i.e. N <= max_depth+1).
        if max_depth >= 0 {
            let depth = rel_canonical.components().count() as u32;
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

/// Lexical hidden check on a request path: any `/`-separated component
/// starting with '.'. Used only as a fast-path pre-filter before touching the
/// filesystem; the authoritative hidden ruling is [`path_has_hidden_component`]
/// on the canonicalized path, which sees the platform's real components.
fn has_hidden_component(rel_path: &str) -> bool {
    rel_path.split('/').any(|component| {
        component.starts_with('.')
    })
}

/// Check if any component of a canonicalized relative path is hidden (its
/// name starts with '.'). Operating on real `Path` components means platform
/// semantics are already baked in: on Windows `\` separators are real
/// components here, and 8.3 short names have been expanded to the real name
/// by canonicalization.
fn path_has_hidden_component(path: &Path) -> bool {
    path.components().any(|component| match component {
        std::path::Component::Normal(name) => name.as_encoded_bytes().starts_with(b"."),
        _ => false,
    })
}

/// Lexical depth of a URL-side relative path (number of non-empty `/`
/// segments). "" → 0, "a" → 1, "a/b" → 2, "a/b/c" → 3
///
/// Used for display and listing decisions only; security rulings measure the
/// canonicalized path's real component count instead (see `safe_resolve`).
fn path_depth(rel_path: &str) -> u32 {
    if rel_path.is_empty() {
        return 0;
    }
    rel_path.split('/').filter(|s| !s.is_empty()).count() as u32
}

/// Percent-encode each non-empty segment of a relative path. Unlike encoding
/// the whole string at once, this preserves `/` separators while encoding
/// characters that are unsafe in a URL path segment (space, `#`, `?`, `%`,
/// non-ASCII, etc.). Used when building hrefs so that parent directory names
/// containing special characters produce valid URIs for WebDAV clients.
pub(crate) fn encode_path_segments(rel: &str) -> String {
    rel.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| percent_encoding::utf8_percent_encode(s, PATH_SEGMENT_ENCODE).to_string())
        .collect::<Vec<_>>()
        .join("/")
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
    // Normalize: strip leading/trailing slashes and collapse runs of '/' so
    // that hrefs and the `path` field are clean regardless of how the caller
    // arrived here (e.g. a browser refresh on `/sub/` would otherwise produce
    // hrefs like `/sub//child` and a path of `/sub/`, which compounds as the
    // user navigates deeper). `safe_resolve` already canonicalized the path
    // against the filesystem, so this is purely cosmetic normalization of the
    // relative-path string used for display and link generation.
    let normalized_rel: String = rel_path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    let rel_path = normalized_rel.as_str();

    let current_depth = path_depth(rel_path);
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
            format!(
                "/{}/{}",
                encode_path_segments(rel),
                percent_encoding::utf8_percent_encode(&name, PATH_SEGMENT_ENCODE)
            )
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

    /// Regression for the `%5C` bypass: on Windows `\` is a real path
    /// separator, so `public\.hidden\secret.txt` used to slip past the
    /// '/'-only lexical hidden check while `join`/`canonicalize` still
    /// descended into `.hidden`. The ruling now happens on the canonicalized
    /// path's real components.
    #[cfg(windows)]
    #[tokio::test]
    async fn safe_resolve_backslash_hidden_denied() {
        let (_tmp, root) = tmp_root();
        fs::create_dir_all(root.join("public/.hidden")).unwrap();
        fs::write(root.join("public/.hidden/secret.txt"), "data").unwrap();

        // Read path: hidden component in the middle of a backslash path.
        assert!(safe_resolve(&root, r"public\.hidden\secret.txt", false, -1).await.is_err());
        // Write paths: hidden component in the parent chain, and as the
        // final (to-be-created) name itself.
        assert!(safe_resolve_parent(&root, r"public\.hidden\new.txt", false, -1).await.is_err());
        assert!(safe_resolve_parent(&root, r"public\.newhidden", false, -1).await.is_err());
        // Sanity: the visible sibling stays reachable with either spelling.
        fs::write(root.join("public/ok.txt"), "ok").unwrap();
        assert!(safe_resolve(&root, r"public\ok.txt", false, -1).await.is_ok());
    }

    /// Same root cause, second ruling: a `\`-separated path used to count as
    /// a single '/'-segment, so `--max-depth` was bypassed on Windows.
    #[cfg(windows)]
    #[tokio::test]
    async fn safe_resolve_backslash_depth_denied() {
        let (_tmp, root) = tmp_root();
        fs::create_dir_all(root.join("a/b/c")).unwrap();
        fs::write(root.join("a/b/file.txt"), "x").unwrap();
        fs::write(root.join("a/b/c/deep.txt"), "x").unwrap();

        // File at depth 4 is over the limit (max_depth 2 allows files to
        // depth 3), whichever separator spelling the request uses.
        assert!(safe_resolve(&root, "a/b/c/deep.txt", false, 2).await.is_err());
        assert!(safe_resolve(&root, r"a\b\c\deep.txt", false, 2).await.is_err());
        // Write side: target whose canonical parent sits at depth 3.
        assert!(safe_resolve_parent(&root, r"a\b\c\new.txt", false, 2).await.is_err());
        // File at depth 3 stays allowed with either spelling.
        assert!(safe_resolve(&root, r"a\b\file.txt", false, 2).await.is_ok());
        assert!(safe_resolve_parent(&root, r"a\b\new.txt", false, 2).await.is_ok());
    }

    /// On Unix, `\` is a legal filename character, NOT a separator: a file
    /// named `a\.hidden` is a single component and does not start with '.',
    /// so it must remain accessible. (Ruling on canonical components keeps
    /// this a non-event; a naive split on both separators would be a false
    /// positive here.)
    #[cfg(unix)]
    #[tokio::test]
    async fn safe_resolve_backslash_is_literal_on_unix() {
        let (_tmp, root) = tmp_root();
        fs::write(root.join(r"a\.hidden"), "data").unwrap();
        assert!(safe_resolve(&root, r"a\.hidden", false, -1).await.is_ok());
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

    #[tokio::test]
    async fn list_directory_href_encodes_parent_segments() {
        let (_tmp, root) = tmp_root();
        fs::create_dir_all(root.join("my dir/#1")).unwrap();
        fs::write(root.join("my dir/#1/child file.txt"), "data").unwrap();

        let listing = list_directory(&root, "my dir/#1", false, -1).await.unwrap();
        let entry = &listing.entries[0];
        // Parent segments (space, '#') must be encoded; only '/' separators remain.
        assert_eq!(entry.href, "/my%20dir/%231/child%20file.txt");
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

    // ─── list_directory path normalization ───

    /// Regression: a trailing slash or doubled separator in the request path
    /// must not leak into the `path` field or the entry `href`s. Before the
    /// fix, `/sub/` produced `path: "/sub/"` and hrefs like `/sub//child`,
    /// which compounded as the user navigated deeper (looking like the folder
    /// structure was being duplicated).
    #[tokio::test]
    async fn list_directory_normalizes_trailing_and_doubled_slashes() {
        let (_tmp, root) = tmp_root();
        fs::create_dir_all(root.join("sub/child")).unwrap();
        fs::write(root.join("sub/inner.txt"), "x").unwrap();

        for bad_rel in ["sub/", "sub//", "/sub", "//sub//", "sub///"] {
            let listing = list_directory(&root, bad_rel, false, -1).await.unwrap();
            assert_eq!(listing.path, "/sub", "rel={bad_rel:?} path");
            let child = listing.entries.iter().find(|e| e.name == "child").unwrap();
            assert_eq!(child.href, "/sub/child", "rel={bad_rel:?} child href");
            let inner = listing.entries.iter().find(|e| e.name == "inner.txt").unwrap();
            assert_eq!(inner.href, "/sub/inner.txt", "rel={bad_rel:?} inner href");
        }
    }
}
