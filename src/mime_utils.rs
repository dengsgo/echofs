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
