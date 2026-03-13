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
    fn detect_mime_mp4() {
        let m = detect_mime(Path::new("video.mp4"));
        assert_eq!(m.type_(), mime::VIDEO);
    }

    #[test]
    fn detect_mime_png() {
        let m = detect_mime(Path::new("image.png"));
        assert_eq!(m, mime::IMAGE_PNG);
    }

    #[test]
    fn detect_mime_txt() {
        let m = detect_mime(Path::new("file.txt"));
        assert_eq!(m, mime::TEXT_PLAIN);
    }

    #[test]
    fn detect_mime_unknown_ext() {
        let m = detect_mime(Path::new("file.xyzabc"));
        assert_eq!(m, mime::APPLICATION_OCTET_STREAM);
    }

    #[test]
    fn detect_mime_no_extension() {
        let m = detect_mime(Path::new("noext"));
        assert_eq!(m, mime::APPLICATION_OCTET_STREAM);
    }

    #[test]
    fn test_is_video() {
        let m = detect_mime(Path::new("video.mp4"));
        assert!(is_video(&m));
        assert!(!is_audio(&m));
        assert!(!is_image(&m));
    }

    #[test]
    fn test_is_audio() {
        let m = detect_mime(Path::new("song.mp3"));
        assert!(is_audio(&m));
        assert!(!is_video(&m));
    }

    #[test]
    fn test_is_image() {
        let m = detect_mime(Path::new("photo.jpg"));
        assert!(is_image(&m));
        assert!(!is_video(&m));
    }

    #[test]
    fn test_is_text() {
        let m = detect_mime(Path::new("readme.txt"));
        assert!(is_text(&m));
    }

    #[test]
    fn test_is_media() {
        assert!(is_media(&detect_mime(Path::new("v.mp4"))));
        assert!(is_media(&detect_mime(Path::new("a.mp3"))));
        assert!(is_media(&detect_mime(Path::new("i.png"))));
        assert!(!is_media(&detect_mime(Path::new("f.txt"))));
    }

    #[test]
    fn icon_directory() {
        assert_eq!(icon_for_path(Path::new("anything"), true), "folder");
    }

    #[test]
    fn icon_video() {
        assert_eq!(icon_for_path(Path::new("v.mp4"), false), "video");
    }

    #[test]
    fn icon_audio() {
        assert_eq!(icon_for_path(Path::new("a.mp3"), false), "audio");
    }

    #[test]
    fn icon_image() {
        assert_eq!(icon_for_path(Path::new("i.jpg"), false), "image");
    }

    #[test]
    fn icon_text() {
        assert_eq!(icon_for_path(Path::new("f.txt"), false), "text");
    }

    #[test]
    fn icon_code() {
        // .rs is detected as text/x-rust by mime_guess, so is_text() matches first.
        // Use an extension that mime_guess doesn't recognize as text, e.g. .go
        assert_eq!(icon_for_path(Path::new("main.go"), false), "code");
    }

    #[test]
    fn icon_archive() {
        assert_eq!(icon_for_path(Path::new("data.zip"), false), "archive");
    }

    #[test]
    fn icon_pdf() {
        assert_eq!(icon_for_path(Path::new("doc.pdf"), false), "pdf");
    }

    #[test]
    fn icon_unknown() {
        assert_eq!(icon_for_path(Path::new("file.xyzabc"), false), "file");
    }
}
