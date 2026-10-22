use super::vdf;
use std::{
    io::Read,
    path::{Component, Path, PathBuf},
};

const CONFIG_LIMIT: u64 = 1024 * 1024;

pub(super) fn directories() -> Vec<PathBuf> {
    let home = if cfg!(windows) {
        std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))
    } else {
        std::env::var_os("HOME")
    }
    .map(PathBuf::from);
    let programs = ["ProgramFiles", "ProgramFiles(x86)"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let (games, mut steam) = roots(std::env::consts::OS, home.as_deref(), &programs);
    #[cfg(windows)]
    if let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey(r"Software\Valve\Steam")
        && let Ok(path) = key.get_value::<String, _>("SteamPath")
    {
        steam.push(PathBuf::from(path));
    }
    steam.sort();
    steam.dedup();
    expand(games, steam)
}

fn roots(
    platform: &str,
    home: Option<&Path>,
    programs: &[PathBuf],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut games = vec![];
    let mut steam = vec![];
    if platform == "macos" {
        games.push(PathBuf::from("/Applications/MTGA.app"));
        for name in ["MTGA", "MagicTheGathering"] {
            games.push(PathBuf::from("/Users/Shared/Epic Games").join(name));
        }
        if let Some(home) = home {
            games.push(home.join("Applications/MTGA.app"));
            steam.push(home.join("Library/Application Support/Steam"));
        }
    } else if platform == "windows" {
        for root in programs {
            steam.push(root.join("Steam"));
            games.push(root.join("Wizards of the Coast/MTGA"));
            games.push(root.join("Epic Games/MTGA"));
            games.push(root.join("Epic Games/MagicTheGathering"));
        }
        if let Some(home) = home {
            games.push(home.join("AppData/Local/Programs/Wizards of the Coast/MTGA"));
        }
    }
    (games, steam)
}

fn values(path: &Path, key: &str) -> Vec<String> {
    let Ok(file) = std::fs::File::open(path) else {
        return vec![];
    };
    let mut source = String::new();
    if file
        .take(CONFIG_LIMIT + 1)
        .read_to_string(&mut source)
        .is_err()
        || source.len() as u64 > CONFIG_LIMIT
    {
        return vec![];
    }
    vdf::values(&source, key)
}

fn expand(mut games: Vec<PathBuf>, steam: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut libraries = steam.clone();
    for root in steam {
        libraries.extend(
            values(&root.join("steamapps/libraryfolders.vdf"), "path")
                .into_iter()
                .map(PathBuf::from)
                .filter(|p| p.is_absolute()),
        );
    }
    libraries.sort();
    libraries.dedup();
    for library in libraries.into_iter().take(128) {
        let folder = values(
            &library.join("steamapps/appmanifest_2141910.acf"),
            "installdir",
        )
        .into_iter()
        .find(|p| {
            !p.is_empty()
                && Path::new(p)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
        })
        .unwrap_or_else(|| "MTGA".into());
        games.push(library.join("steamapps/common").join(folder));
    }
    let mut directories = vec![];
    for game in games {
        for tail in [
            "MTGA_Data/Downloads/Raw",
            "Contents/Resources/Data/Downloads/Raw",
            "MTGA.app/Contents/Resources/Data/Downloads/Raw",
        ] {
            directories.push(game.join(tail));
        }
    }
    directories.sort();
    directories.dedup();
    directories
}

#[cfg(test)]
#[path = "../../tests/unit/card_locations.rs"]
mod tests;
