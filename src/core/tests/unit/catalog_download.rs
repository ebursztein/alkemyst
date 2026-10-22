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

async fn body(bytes: &[u8]) -> reqwest::Response {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let bytes = bytes.to_vec();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut header = Vec::new();
        while !header.ends_with(b"\r\n\r\n") {
            header.push(socket.read_u8().await.unwrap());
            assert!(header.len() < 8192);
        }
        socket
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    bytes.len()
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        socket.write_all(&bytes).await.unwrap();
    });
    client()
        .unwrap()
        .get(format!("http://{address}/archive"))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn archive_size_is_checked_before_decompression() {
    let mut output = Vec::new();
    copy_archive(body(b"abcd").await, &mut output, 4)
        .await
        .unwrap();
    assert_eq!(output, b"abcd");
    assert!(
        copy_archive(body(b"abc").await, &mut Vec::new(), 4)
            .await
            .is_err()
    );
    assert!(
        copy_archive(body(b"abcde").await, &mut Vec::new(), 4)
            .await
            .is_err()
    );
}
