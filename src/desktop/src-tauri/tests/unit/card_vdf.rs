use super::values;

#[test]
fn paths_preserve_spaces_unicode_and_escaped_windows_separators() {
    assert_eq!(
        values(
            r#""libraryfolders" { "0" { "path" "C:\\Steam Library" } "1" { "path" "/Volumes/Jeux Été" } }"#,
            "path"
        ),
        [r"C:\Steam Library", "/Volumes/Jeux Été"]
    );
    assert_eq!(
        values(r#"path "a\"b" PATH "D:\Games""#, "path"),
        ["a\"b", r"D:\Games"]
    );
}

#[test]
fn comments_boundaries_truncation_and_control_characters_never_supply_paths() {
    assert_eq!(
        values(
            r#""label" "path" "/not-a-path-key" "ignored" "path" "/real""#,
            "path"
        ),
        ["/real"]
    );
    assert_eq!(
        values(
            "\u{feff}// \"path\" \"fake\"\n/* path bad */ path real",
            "path"
        ),
        ["real"]
    );
    assert!(values(r#""path" { "not a value" }"#, "path").is_empty());
    assert!(values("\"path\" \"unterminated", "path").is_empty());
    assert!(values("path \"bad\0path\"", "path").is_empty());
    assert!(values(&format!("path \"{}\"", "x".repeat(32_769)), "path").is_empty());
    assert!(values(&format!("path {}", "x".repeat(32_769)), "path").is_empty());
}
