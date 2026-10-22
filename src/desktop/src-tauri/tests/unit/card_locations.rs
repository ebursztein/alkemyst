use super::*;

#[test]
fn platform_roots_include_standard_steam_and_standalone_installations() {
    let home = Path::new("home");
    let (games, steam) = roots("macos", Some(home), &[]);
    assert!(games.contains(&PathBuf::from("/Applications/MTGA.app")));
    assert!(steam.contains(&home.join("Library/Application Support/Steam")));
    let programs = PathBuf::from("programs");
    let (games, steam) = roots("windows", Some(home), std::slice::from_ref(&programs));
    assert!(games.contains(&programs.join("Wizards of the Coast/MTGA")));
    assert!(steam.contains(&programs.join("Steam")));
    assert!(games.contains(&home.join("AppData/Local/Programs/Wizards of the Coast/MTGA")));
    assert_eq!(roots("unknown", None, &[]), (vec![], vec![]));
}

#[test]
fn custom_steam_libraries_and_manifest_folders_are_followed_without_recursive_scanning() {
    let dir = tempfile::tempdir().unwrap();
    let steam = dir.path().join("Steam");
    let custom = dir.path().join("Custom Library");
    std::fs::create_dir_all(steam.join("steamapps")).unwrap();
    std::fs::create_dir_all(custom.join("steamapps")).unwrap();
    std::fs::write(
        steam.join("steamapps/libraryfolders.vdf"),
        format!(
            "\"libraryfolders\" {{ \"0\" {{ \"path\" {} }} }}",
            serde_json::to_string(&custom).unwrap()
        ),
    )
    .unwrap();
    std::fs::write(
        custom.join("steamapps/appmanifest_2141910.acf"),
        "\"AppState\" { \"installdir\" \"Arena Custom\" }",
    )
    .unwrap();
    let directories = expand(vec![], vec![steam]);
    assert!(
        directories.contains(&custom.join("steamapps/common/Arena Custom/MTGA_Data/Downloads/Raw"))
    );
    std::fs::write(
        custom.join("steamapps/appmanifest_2141910.acf"),
        "\"installdir\" \"../escape\"",
    )
    .unwrap();
    let directories = expand(vec![], vec![custom.clone()]);
    assert!(directories.contains(&custom.join("steamapps/common/MTGA/MTGA_Data/Downloads/Raw")));
    let large = dir.path().join("large.vdf");
    std::fs::write(&large, vec![b'x'; (CONFIG_LIMIT + 1) as usize]).unwrap();
    assert!(values(&large, "path").is_empty());
}
