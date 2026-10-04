use crate::domain::errors::AppError;

#[derive(Debug, Clone, Copy)]
pub struct MediaSpec {
    pub media_type: &'static str,
    pub max_size: usize,
}

pub fn spec_for(mime: &str, extension: &str) -> Result<MediaSpec, AppError> {
    let extension = extension.to_ascii_lowercase();
    let spec = match (mime, extension.as_str()) {
        ("image/jpeg", "jpg" | "jpeg")
        | ("image/png", "png")
        | ("image/gif", "gif")
        | ("image/webp", "webp")
        | ("image/heic", "heic") => MediaSpec {
            media_type: "image",
            max_size: 10 * 1024 * 1024,
        },
        ("video/mp4", "mp4") | ("video/quicktime", "mov") | ("video/webm", "webm") => MediaSpec {
            media_type: "video",
            max_size: 50 * 1024 * 1024,
        },
        ("application/pdf", "pdf")
        | ("application/msword", "doc")
        | ("application/vnd.openxmlformats-officedocument.wordprocessingml.document", "docx")
        | ("application/vnd.ms-excel", "xls")
        | ("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "xlsx")
        | ("application/vnd.ms-powerpoint", "ppt")
        | ("application/vnd.openxmlformats-officedocument.presentationml.presentation", "pptx")
        | ("text/plain", "txt")
        | ("text/csv", "csv") => MediaSpec {
            media_type: "document",
            max_size: 25 * 1024 * 1024,
        },
        ("audio/ogg", "ogg")
        | ("audio/opus", "opus")
        | ("audio/mp4", "m4a")
        | ("audio/webm", "webm") => MediaSpec {
            media_type: "voice",
            max_size: 10 * 1024 * 1024,
        },
        _ => {
            return Err(AppError::Validation(
                "Unsupported media type or extension".into(),
            ))
        }
    };
    Ok(spec)
}

pub fn validate_magic(mime: &str, bytes: &[u8]) -> Result<(), AppError> {
    let valid = match mime {
        "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "image/webp" => bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "image/heic" => {
            is_iso_bmff(bytes, b"heic")
                || is_iso_bmff(bytes, b"heix")
                || is_iso_bmff(bytes, b"mif1")
        }
        "video/mp4" => {
            is_iso_bmff(bytes, b"mp41")
                || is_iso_bmff(bytes, b"mp42")
                || is_iso_bmff(bytes, b"isom")
                || is_iso_bmff(bytes, b"iso2")
        }
        "video/quicktime" => is_iso_bmff(bytes, b"qt  "),
        "video/webm" => bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]),
        "application/pdf" => bytes.starts_with(b"%PDF-"),
        "application/msword" | "application/vnd.ms-excel" | "application/vnd.ms-powerpoint" => {
            bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
        }
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        | "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            bytes.starts_with(b"PK")
        }
        "text/plain" | "text/csv" => bytes
            .iter()
            .all(|b| *b == b'\n' || *b == b'\r' || *b == b'\t' || *b >= 0x20),
        "audio/ogg" | "audio/opus" => bytes.starts_with(b"OggS"),
        "audio/mp4" => is_iso_bmff(bytes, b"M4A ") || is_iso_bmff(bytes, b"mp42"),
        "audio/webm" => bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]),
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(AppError::Validation("INVALID_FILE_TYPE".into()))
    }
}

fn is_iso_bmff(bytes: &[u8], brand: &[u8; 4]) -> bool {
    bytes.len() >= 12 && &bytes[4..8] == b"ftyp" && &bytes[8..12] == brand
}

pub fn sanitize_filename(value: &str) -> String {
    let name = value.rsplit(['/', '\\']).next().unwrap_or("file");
    let mut output = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect::<String>();
    if output.len() > 255 {
        output.truncate(255);
    }
    if output.is_empty() {
        "file".into()
    } else {
        output
    }
}

pub fn extension(filename: &str) -> Result<&str, AppError> {
    filename
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| !ext.is_empty())
        .ok_or_else(|| AppError::Validation("File extension is required".into()))
}
