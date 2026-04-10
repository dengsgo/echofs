use std::path::Path;

pub fn detect_mime(path: &Path) -> mime::Mime {
    mime_guess::from_path(path)
        .first()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM)
}

pub fn is_video(m: &mime::Mime) -> bool {
    m.type_() == mime::VIDEO
}

pub fn is_audio(m: &mime::Mime) -> bool {
    m.type_() == mime::AUDIO
}

pub fn is_image(m: &mime::Mime) -> bool {
    m.type_() == mime::IMAGE
}

pub fn is_text(m: &mime::Mime) -> bool {
    m.type_() == mime::TEXT
}

pub fn is_media(m: &mime::Mime) -> bool {
    is_video(m) || is_audio(m) || is_image(m)
}

pub fn icon_for_path(path: &Path, is_dir: bool) -> &'static str {
    if is_dir {
        return "folder";
    }
    let m = detect_mime(path);
    if is_video(&m) {
        "video"
    } else if is_audio(&m) {
        "audio"
    } else if is_image(&m) {
        "image"
    } else if is_text(&m) {
        "text"
    } else {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
            .as_str()
        {
            "pdf" => "pdf",
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "archive",
            "doc" | "docx" | "odt" | "rtf" => "document",
            "xls" | "xlsx" | "ods" | "csv" => "spreadsheet",
            "ppt" | "pptx" | "odp" => "presentation",
            "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "rb" | "php"
            | "swift" | "kt" | "sh" | "bash" | "zsh" | "json" | "yaml" | "yml" | "toml"
            | "xml" | "html" | "css" | "sql" => "code",
            _ => "file",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detect_mime_types() {
        // (filename, expected_type_str)
        let cases: Vec<(&str, &str)> = vec![
            ("video.mp4",    "video"),
            ("image.png",    "image"),
            ("file.txt",     "text"),
            ("file.xyzabc",  "application"),
            ("noext",        "application"),
        ];
        for (file, expected_type) in cases {
            let m = detect_mime(Path::new(file));
            assert_eq!(m.type_().as_str(), expected_type, "MIME type mismatch for {}", file);
        }
        // Exact MIME checks
        assert_eq!(detect_mime(Path::new("image.png")), mime::IMAGE_PNG);
        assert_eq!(detect_mime(Path::new("file.txt")), mime::TEXT_PLAIN);
        assert_eq!(detect_mime(Path::new("file.xyzabc")), mime::APPLICATION_OCTET_STREAM);
    }

    #[test]
    fn media_type_classification() {
        // (filename, is_video, is_audio, is_image, is_text, is_media)
        let cases = [
            ("video.mp4", true,  false, false, false, true),
            ("song.mp3",  false, true,  false, false, true),
            ("photo.jpg", false, false, true,  false, true),
            ("readme.txt",false, false, false, true,  false),
        ];
        for (file, vid, aud, img, txt, med) in cases {
            let m = detect_mime(Path::new(file));
            assert_eq!(is_video(&m), vid, "is_video mismatch for {}", file);
            assert_eq!(is_audio(&m), aud, "is_audio mismatch for {}", file);
            assert_eq!(is_image(&m), img, "is_image mismatch for {}", file);
            assert_eq!(is_text(&m),  txt, "is_text mismatch for {}", file);
            assert_eq!(is_media(&m), med, "is_media mismatch for {}", file);
        }
    }

    #[test]
    fn icon_mapping() {
        // (filename, is_dir, expected_icon)
        let cases = [
            ("anything",    true,  "folder"),
            ("v.mp4",       false, "video"),
            ("a.mp3",       false, "audio"),
            ("i.jpg",       false, "image"),
            ("f.txt",       false, "text"),
            ("main.go",     false, "code"),    // .go not detected as text by mime_guess
            ("data.zip",    false, "archive"),
            ("doc.pdf",     false, "pdf"),
            ("file.xyzabc", false, "file"),
        ];
        for (file, is_dir, expected) in cases {
            assert_eq!(icon_for_path(Path::new(file), is_dir), expected,
                "icon mismatch for {} (is_dir={})", file, is_dir);
        }
    }
}
