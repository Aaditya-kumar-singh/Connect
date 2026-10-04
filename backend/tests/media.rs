use ybm_connect::media::validation;

#[test]
fn rejects_renamed_executable_as_jpeg() {
    assert!(validation::spec_for("image/jpeg", "jpg").is_ok());
    assert!(validation::validate_magic("image/jpeg", b"MZ\x90\x00").is_err());
}

#[test]
fn accepts_real_image_signatures() {
    assert!(validation::validate_magic("image/jpeg", &[0xFF, 0xD8, 0xFF, 0xE0]).is_ok());
    assert!(validation::validate_magic("image/png", b"\x89PNG\r\n\x1a\n").is_ok());
    assert!(validation::validate_magic("image/webp", b"RIFFxxxxWEBP").is_ok());
}

#[test]
fn enforces_media_size_classes() {
    assert_eq!(
        validation::spec_for("image/jpeg", "jpg").unwrap().max_size,
        10 * 1024 * 1024
    );
    assert_eq!(
        validation::spec_for("video/mp4", "mp4").unwrap().max_size,
        50 * 1024 * 1024
    );
    assert_eq!(
        validation::spec_for("application/pdf", "pdf")
            .unwrap()
            .max_size,
        25 * 1024 * 1024
    );
    assert_eq!(
        validation::spec_for("audio/ogg", "ogg").unwrap().max_size,
        10 * 1024 * 1024
    );
}

#[test]
fn sanitizes_paths_and_unsupported_extensions() {
    assert_eq!(
        validation::sanitize_filename(r"C:\temp\my file!.jpg"),
        "myfile.jpg"
    );
    assert!(validation::spec_for("application/octet-stream", "exe").is_err());
}

#[test]
fn rejects_magic_mismatch() {
    assert!(validation::validate_magic("image/png", b"MZfake").is_err());
}
