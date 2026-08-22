//! Streaming ZIP archives for folder download (`GET /path/?download=zip`).
//!
//! The archive is built on a blocking thread that walks the folder and writes
//! into an OS pipe; the async side streams the read end as the response body.
//! The recursive walk is collected up front (entry metadata only, not file
//! contents), so walk failures still produce a proper HTTP error instead of a
//! truncated archive. From there the response flows incrementally — the first
//! bytes reach the client while later files are still being compressed, and
//! transfer memory stays O(pipe buffer) regardless of folder size.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use axum::body::Body;

use crate::error::AppError;
use crate::range;

/// Extensions whose contents are already compressed — deflating them again
/// burns CPU for ~0% size reduction, so they are stored verbatim.
const STORED_EXTENSIONS: &[&str] = &[
    "zip", "gz", "tgz", "bz2", "xz", "zst", "lz4", "7z", "rar", "jar", "apk",
    "jpg", "jpeg", "png", "gif", "webp", "avif", "heic", "heif", "ico",
    "mp4", "mkv", "mov", "avi", "webm", "m4v", "flv", "wmv",
    "mp3", "aac", "flac", "ogg", "opus", "m4a", "wma",
    "pdf", "docx", "xlsx", "pptx", "odt", "ods", "odp", "epub",
    "woff", "woff2", "br", "dmg", "iso",
];

/// One filesystem object to place in the archive.
struct ZipEntry {
    /// Absolute path on disk (canonicalized where symlinks are involved).
    disk: PathBuf,
    /// Path inside the archive: '/'-separated, rooted at the top folder name.
    archive: String,
    is_dir: bool,
    size: u64,
    modified: Option<std::time::SystemTime>,
}

/// Build a streaming response body containing `dir` as a ZIP archive, plus the
/// archive's top-level folder name (for the download filename).
///
/// `rel_from_root` is `dir`'s path relative to `root` ("" when archiving the
/// root itself); it anchors the `--max-depth` accounting so zipped content is
/// limited to exactly what the server would also serve directly.
///
/// With `head_only` (a HEAD request) the walk and compression are skipped
/// entirely and the body is empty — download managers probing with HEAD
/// shouldn't spin up a full archive pass.
pub async fn zip_dir_body(
    dir: PathBuf,
    root: &Path,
    rel_from_root: &str,
    show_hidden: bool,
    max_depth: i32,
    speed_limit: Option<u64>,
    head_only: bool,
) -> Result<(Body, String), AppError> {
    let root = root.to_path_buf();
    let base_depth = rel_from_root
        .split('/')
        .filter(|s| !s.is_empty())
        .count() as u32;

    // Walk happens before the response starts, so failures here still produce
    // a proper HTTP error instead of a truncated archive.
    let (entries, top_name) = tokio::task::spawn_blocking(move || {
        let canonical_root = std::fs::canonicalize(&root)
            .map_err(|_| AppError::Internal("Root directory does not exist".into()))?;
        // Canonicalize the start dir too: `visited` loop-guard compares
        // canonical paths, and callers archiving the root pass it unresolved.
        let dir = std::fs::canonicalize(&dir)
            .map_err(|_| AppError::NotFound("Path not found".into()))?;
        let top_name = top_folder_name(&dir);
        if head_only {
            return Ok::<_, AppError>((Vec::new(), top_name));
        }
        let entries = collect_entries(
            &dir,
            &canonical_root,
            base_depth,
            &top_name,
            show_hidden,
            max_depth,
        )?;
        Ok::<_, AppError>((entries, top_name))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    if head_only {
        return Ok((Body::empty(), top_name));
    }

    let (reader, writer) = std::io::pipe().map_err(AppError::from)?;

    // Producer: compress entries into the pipe on a blocking thread. If the
    // client disconnects, the read end closes and the next write fails with
    // EPIPE, which aborts the archive — no cancellation plumbing needed.
    tokio::task::spawn_blocking(move || {
        if let Err(e) = write_archive(&entries, writer) {
            // Mid-stream failures can't change the already-sent status; the
            // client sees a truncated archive. Broken pipe = client went away.
            // (zip's own Drop also logs "failed to finalize archive" on this
            // path — same event from inside the crate, not a second error.)
            if e.kind() != std::io::ErrorKind::BrokenPipe {
                eprintln!("zip: archive aborted: {}", e);
            }
        }
    });

    let body = range::stream_body(tokio::fs::File::from_std(pipe_reader_to_file(reader)), speed_limit);
    Ok((body, top_name))
}

/// Archive top-level folder name for `dir` (already canonicalized).
fn top_folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "archive".to_string())
}

/// Hand the pipe's read end to tokio. `tokio::fs::File` performs reads on the
/// blocking threadpool, which works for pipe fds/handles on every platform.
#[cfg(unix)]
fn pipe_reader_to_file(reader: std::io::PipeReader) -> std::fs::File {
    std::fs::File::from(std::os::fd::OwnedFd::from(reader))
}

/// Windows counterpart of [`pipe_reader_to_file`].
#[cfg(windows)]
fn pipe_reader_to_file(reader: std::io::PipeReader) -> std::fs::File {
    std::fs::File::from(std::os::windows::io::OwnedHandle::from(reader))
}

/// Recursively collect `dir`'s contents. Security invariants mirror
/// `safe_resolve` for every entry, not just the top folder:
/// - hidden (`.`-prefixed) names are skipped unless `show_hidden`
/// - symlinks are followed only when their canonical target stays inside
///   `canonical_root` (blocks escape via planted links), and already-visited
///   directories are not re-entered (blocks symlink loops)
/// - `--max-depth` uses the same rule as `safe_resolve`: directories at most
///   `max_depth` below root, files at most `max_depth + 1`
fn collect_entries(
    dir: &Path,
    canonical_root: &Path,
    base_depth: u32,
    top_name: &str,
    show_hidden: bool,
    max_depth: i32,
) -> Result<Vec<ZipEntry>, AppError> {
    let mut entries = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    visited.insert(dir.to_path_buf());

    let top_meta = std::fs::metadata(dir).map_err(AppError::from)?;
    entries.push(ZipEntry {
        disk: dir.to_path_buf(),
        archive: top_name.to_string(),
        is_dir: true,
        size: 0,
        modified: top_meta.modified().ok(),
    });

    walk_dir(
        dir,
        canonical_root,
        top_name,
        base_depth,
        show_hidden,
        max_depth,
        &mut entries,
        &mut visited,
    )?;
    Ok(entries)
}

#[allow(clippy::too_many_arguments)]
fn walk_dir(
    cur: &Path,
    canonical_root: &Path,
    archive_prefix: &str,
    cur_root_depth: u32,
    show_hidden: bool,
    max_depth: i32,
    entries: &mut Vec<ZipEntry>,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), AppError> {
    // Sort by name so archive layout is deterministic regardless of the
    // order the filesystem hands back directory entries.
    let mut children: Vec<_> = std::fs::read_dir(cur)
        .map_err(AppError::from)?
        .collect::<Result<_, _>>()
        .map_err(AppError::from)?;
    children.sort_by_key(|e| e.file_name());

    for entry in children {
        let name = entry.file_name().to_string_lossy().into_owned();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let file_type = entry.file_type().map_err(AppError::from)?;
        let (disk, meta) = if file_type.is_symlink() {
            // Follow the link, but only if it lands inside the served root.
            let canonical = match std::fs::canonicalize(entry.path()) {
                Ok(p) => p,
                Err(_) => continue, // dangling link
            };
            if !canonical.starts_with(canonical_root) {
                continue;
            }
            let meta = match std::fs::metadata(&canonical) {
                Ok(m) => m,
                Err(_) => continue,
            };
            (canonical, meta)
        } else {
            (entry.path(), entry.metadata().map_err(AppError::from)?)
        };

        let is_dir = meta.is_dir();
        if !is_dir && !meta.is_file() {
            continue; // sockets, fifos, devices — not archivable
        }

        let root_depth = cur_root_depth + 1;
        if max_depth >= 0 {
            let limit = if is_dir {
                max_depth as u32
            } else {
                max_depth as u32 + 1
            };
            if root_depth > limit {
                continue;
            }
        }

        let archive = format!("{}/{}", archive_prefix, name);
        if is_dir {
            if !visited.insert(disk.clone()) {
                continue; // symlink loop — already archived this tree
            }
            entries.push(ZipEntry {
                disk: disk.clone(),
                archive: archive.clone(),
                is_dir: true,
                size: 0,
                modified: meta.modified().ok(),
            });
            // A subtree that vanishes or loses read permission mid-walk is
            // skipped rather than failing the whole download.
            if let Err(e) = walk_dir(
                &disk,
                canonical_root,
                &archive,
                root_depth,
                show_hidden,
                max_depth,
                entries,
                visited,
            ) {
                eprintln!("zip: skipping {}: {}", disk.display(), e);
            }
        } else {
            entries.push(ZipEntry {
                disk,
                archive,
                is_dir: false,
                size: meta.len(),
                modified: meta.modified().ok(),
            });
        }
    }
    Ok(())
}

/// Write all entries as a ZIP archive into `writer` (the pipe's write end).
/// Uses `new_stream`, which emits data descriptors (CRC/sizes trail each
/// file's data) instead of seeking back to patch headers — this is what makes
/// writing to an unseekable pipe possible.
fn write_archive(entries: &[ZipEntry], writer: impl std::io::Write) -> std::io::Result<()> {
    // `set_auto_large_file`: if a file grows past 4 GiB while being archived
    // (walk-time size said small), the crate writes a Zip64 data descriptor
    // instead of failing the whole stream.
    let mut zip = zip::ZipWriter::new_stream(writer).set_auto_large_file();
    for e in entries {
        let mut opts = zip::write::SimpleFileOptions::default();
        if let Some(dt) = e.modified.and_then(system_time_to_zip_datetime) {
            opts = opts.last_modified_time(dt);
        }
        if e.is_dir {
            // Stream mode (`new_stream`) marks every local header with
            // "data descriptor follows", but `add_directory` never writes
            // the descriptor — strict readers (macOS Archive Utility/ditto)
            // reject such archives. A zero-length Stored entry whose name
            // ends in '/' IS the ZIP convention for directories, and this
            // path does emit the descriptor. 0o40755 = dir bit + rwxr-xr-x.
            let dir_name = format!("{}/", e.archive);
            zip.start_file(
                dir_name,
                opts.compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o40755),
            )?;
            continue;
        }
        if is_already_compressed(&e.archive) {
            opts = opts.compression_method(zip::CompressionMethod::Stored);
        }
        // Zip64 for >4 GiB files (also raised implicitly for huge archives).
        opts = opts.large_file(e.size > 0xFFFF_FFFF);

        // A file that vanished since the walk is skipped; the archive stays
        // valid, just shorter than advertised nowhere (no index is pre-sent).
        let mut file = match std::fs::File::open(&e.disk) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("zip: skipping {}: {}", e.disk.display(), err);
                continue;
            }
        };
        zip.start_file(&e.archive, opts)?;
        std::io::copy(&mut file, &mut zip)?;
    }
    zip.finish()?;
    Ok(())
}

fn is_already_compressed(name: &str) -> bool {
    let Some(ext) = name.rsplit('.').next() else {
        return false;
    };
    let ext = ext.to_ascii_lowercase();
    ext != name && STORED_EXTENSIONS.contains(&ext.as_str())
}

/// ZIP timestamps are MS-DOS local civil time; convert via chrono (already a
/// dependency) so extracted files show the same times the server displays.
fn system_time_to_zip_datetime(t: std::time::SystemTime) -> Option<zip::DateTime> {
    use chrono::{Datelike, Timelike};
    let lt: chrono::DateTime<chrono::Local> = t.into();
    let year = u16::try_from(lt.year()).ok()?;
    zip::DateTime::from_date_and_time(
        year,
        lt.month() as u8,
        lt.day() as u8,
        lt.hour() as u8,
        lt.minute() as u8,
        lt.second() as u8,
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn write_file(path: &Path, content: &[u8]) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, content).expect("write");
    }

    fn names(entries: &[ZipEntry]) -> Vec<String> {
        entries.iter().map(|e| e.archive.clone()).collect()
    }

    #[test]
    fn collects_nested_tree_with_top_folder_prefix() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        write_file(&root.join("docs/a.txt"), b"a");
        write_file(&root.join("docs/sub/b.txt"), b"b");

        let dir = std::fs::canonicalize(root.join("docs")).expect("canonical");
        let root_c = std::fs::canonicalize(root).expect("canonical root");
        let entries = collect_entries(&dir, &root_c, 1, "docs", false, -1).expect("collect");

        assert_eq!(
            names(&entries),
            vec!["docs", "docs/a.txt", "docs/sub", "docs/sub/b.txt"]
        );
        assert!(entries[0].is_dir && entries[2].is_dir);
        assert!(!entries[1].is_dir && entries[1].size == 1);
    }

    #[test]
    fn hidden_entries_respect_show_hidden() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        write_file(&root.join("docs/.secret"), b"s");
        write_file(&root.join("docs/visible.txt"), b"v");

        let dir = std::fs::canonicalize(root.join("docs")).expect("canonical");
        let root_c = std::fs::canonicalize(root).expect("canonical root");

        let hidden_off = collect_entries(&dir, &root_c, 1, "docs", false, -1).expect("collect");
        assert_eq!(names(&hidden_off), vec!["docs", "docs/visible.txt"]);

        let hidden_on = collect_entries(&dir, &root_c, 1, "docs", true, -1).expect("collect");
        assert!(names(&hidden_on).contains(&"docs/.secret".to_string()));
    }

    #[test]
    fn max_depth_limits_recursion() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        // docs is at depth 1; docs/deep at depth 2; file at depth 3.
        write_file(&root.join("docs/deep/x.txt"), b"x");

        let dir = std::fs::canonicalize(root.join("docs")).expect("canonical");
        let root_c = std::fs::canonicalize(root).expect("canonical root");
        // max_depth = 1: dir at depth 2 is over the limit → excluded entirely.
        let entries = collect_entries(&dir, &root_c, 1, "docs", false, 1).expect("collect");
        assert_eq!(names(&entries), vec!["docs"]);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_escaping_root_are_skipped() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path().join("root");
        let outside = tmp.path().join("outside");
        std::fs::create_dir_all(root.join("docs")).expect("mkdir");
        std::fs::create_dir_all(&outside).expect("mkdir");
        write_file(&outside.join("secret.txt"), b"s");
        std::os::unix::fs::symlink(&outside, root.join("docs/escape")).expect("symlink");
        write_file(&root.join("docs/ok.txt"), b"o");

        let dir = std::fs::canonicalize(root.join("docs")).expect("canonical");
        let root_c = std::fs::canonicalize(&root).expect("canonical root");
        let entries = collect_entries(&dir, &root_c, 1, "docs", false, -1).expect("collect");
        assert_eq!(names(&entries), vec!["docs", "docs/ok.txt"]);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loops_do_not_recurse_forever() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        write_file(&root.join("docs/f.txt"), b"f");
        // docs/loop -> docs (ancestor): canonical target is inside root.
        std::os::unix::fs::symlink(root.join("docs"), root.join("docs/loop")).expect("symlink");

        let dir = std::fs::canonicalize(root.join("docs")).expect("canonical");
        let root_c = std::fs::canonicalize(root).expect("canonical root");
        let entries = collect_entries(&dir, &root_c, 1, "docs", false, -1).expect("collect");
        assert_eq!(names(&entries), vec!["docs", "docs/f.txt"]);
    }

    #[test]
    fn archive_bytes_form_a_valid_zip() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        write_file(&root.join("docs/hello.txt"), b"hello world");
        write_file(&root.join("docs/photo.jpg"), b"\xFF\xD8\xFF");

        let dir = std::fs::canonicalize(root.join("docs")).expect("canonical");
        let root_c = std::fs::canonicalize(root).expect("canonical root");
        let entries = collect_entries(&dir, &root_c, 1, "docs", false, -1).expect("collect");

        let mut buf: Vec<u8> = Vec::new();
        write_archive(&entries, &mut buf).expect("write archive");

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(buf)).expect("parse zip");
        let mut found: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).expect("entry").name().to_string())
            .collect();
        found.sort();
        assert_eq!(found, vec!["docs/", "docs/hello.txt", "docs/photo.jpg"]);

        let mut content = String::new();
        archive
            .by_name("docs/hello.txt")
            .expect("entry")
            .read_to_string(&mut content)
            .expect("read");
        assert_eq!(content, "hello world");

        // Pre-compressed formats must be stored, not deflated.
        let jpg = archive.by_name("docs/photo.jpg").expect("entry");
        assert_eq!(jpg.compression(), zip::CompressionMethod::Stored);
    }

    /// Regression: in stream mode every local header carries the
    /// "data descriptor follows" flag, so EVERY entry — directories and
    /// zero-length files included — must be trailed by a `PK\x07\x08`
    /// descriptor. zip's `add_directory` skips it (breaking strict readers
    /// like macOS Archive Utility); the trailing-slash Stored-entry approach
    /// used here must not regress back.
    #[test]
    fn every_streamed_entry_is_followed_by_a_data_descriptor() {
        let tmp = tempfile::tempdir().expect("tmp");
        let root = tmp.path();
        write_file(&root.join("d/empty.txt"), b"");
        write_file(&root.join("d/f.txt"), b"data");

        let dir = std::fs::canonicalize(root.join("d")).expect("canonical");
        let root_c = std::fs::canonicalize(root).expect("canonical root");
        let entries = collect_entries(&dir, &root_c, 1, "d", false, -1).expect("collect");
        assert_eq!(entries.len(), 3); // d/ + d/empty.txt + d/f.txt

        let mut buf: Vec<u8> = Vec::new();
        write_archive(&entries, &mut buf).expect("write archive");

        // (Test contents are tiny ASCII, so signature bytes can't appear
        // anywhere but the real structures.)
        let count = |sig: &[u8]| {
            buf.windows(sig.len())
                .filter(|w| *w == sig)
                .count()
        };
        assert_eq!(count(b"PK\x03\x04"), 3, "local headers");
        assert_eq!(count(b"PK\x07\x08"), 3, "data descriptors");
        assert_eq!(count(b"PK\x01\x02"), 3, "central directory entries");
    }
}
