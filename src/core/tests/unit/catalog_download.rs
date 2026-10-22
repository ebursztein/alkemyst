use super::*;
use serde_json::json;

fn manifest() -> Value {
    json!({"type":"default_cards", "updated_at":"2026-10-21T21:00:00Z",
        "jsonl_download_uri":"https://data.scryfall.io/default-cards/example.jsonl.gz", "compressed_size":100})
}

#[test]
fn manifest_requires_official_https_source_and_bounded_size() {
    assert!(descriptor(manifest()).is_ok());
    for (key, value) in [
        ("type", json!("all_cards")),
        ("jsonl_download_uri", json!(null)),
        ("updated_at", json!(null)),
        ("updated_at", json!("")),
        ("compressed_size", json!(null)),
        ("compressed_size", json!(0)),
        ("compressed_size", json!(MAX_ARCHIVE + 1)),
        ("jsonl_download_uri", json!("broken")),
        (
            "jsonl_download_uri",
            json!("https://data.scryfall.io.evil.test/"),
        ),
        ("jsonl_download_uri", json!("http://data.scryfall.io/")),
        (
            "jsonl_download_uri",
            json!("https://user@data.scryfall.io/"),
        ),
        (
            "jsonl_download_uri",
            json!("https://data.scryfall.io:8443/"),
        ),
    ] {
        let mut changed = manifest();
        changed[key] = value;
        assert!(descriptor(changed).is_err());
    }
}

#[test]
fn archive_size_is_checked_before_decompression() {
    let mut output = Vec::new();
    copy_archive(&b"abcd"[..], &mut output, 4).unwrap();
    assert_eq!(output, b"abcd");
    assert!(copy_archive(&b"abc"[..], &mut Vec::new(), 4).is_err());
    assert!(copy_archive(&b"abcde"[..], &mut Vec::new(), 4).is_err());
}
