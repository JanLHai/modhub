use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use super::{Game, GameScanner, GameSource, ScanResult};

#[derive(Debug, Default)]
pub struct WindowsGameScanner;

impl GameScanner for WindowsGameScanner {
    fn scan(&self) -> ScanResult {
        let mut result = ScanResult::default();

        scan_steam(&mut result);
        scan_epic_games(&mut result);
        scan_uninstall_registry(&mut result);

        result
            .games
            .sort_by_key(|game| game.name.to_ascii_lowercase());
        deduplicate_games(&mut result.games);
        result
    }
}

fn scan_steam(result: &mut ScanResult) {
    let mut steam_roots = Vec::new();
    for key in [
        r"HKCU\Software\Valve\Steam",
        r"HKLM\SOFTWARE\Valve\Steam",
        r"HKLM\SOFTWARE\WOW6432Node\Valve\Steam",
    ] {
        if let Some(path) = registry_value(key, "SteamPath") {
            steam_roots.push(path);
        }
    }

    for steam_root in steam_roots {
        let mut libraries = vec![steam_root.clone()];
        let library_file = steam_root.join("steamapps").join("libraryfolders.vdf");

        if let Ok(contents) = fs::read_to_string(library_file) {
            libraries.extend(
                contents
                    .lines()
                    .filter_map(|line| parse_quoted_value(line, "path"))
                    .map(|path| PathBuf::from(unescape_vdf(&path))),
            );
        }

        for library in libraries {
            let manifests = library.join("steamapps");
            let entries = match fs::read_dir(&manifests) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                    continue;
                }

                let Ok(contents) = fs::read_to_string(entry.path()) else {
                    continue;
                };
                let Some(name) = find_vdf_value(&contents, "name") else {
                    continue;
                };
                let Some(install_dir) = find_vdf_value(&contents, "installdir") else {
                    continue;
                };
                if !is_likely_steam_game(&name) {
                    continue;
                }

                let path = manifests.join("common").join(unescape_vdf(&install_dir));
                add_game(result, name, path, GameSource::Steam);
            }
        }
    }
}

fn scan_epic_games(result: &mut ScanResult) {
    let Some(program_data) = std::env::var_os("ProgramData") else {
        return;
    };
    let manifest_dir = PathBuf::from(program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");

    let Ok(entries) = fs::read_dir(manifest_dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("item")
        {
            continue;
        }

        let Ok(contents) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Some(name) = parse_json_string(&contents, "DisplayName") else {
            continue;
        };
        let Some(path) = parse_json_string(&contents, "InstallLocation") else {
            continue;
        };

        add_game(result, name, PathBuf::from(path), GameSource::EpicGames);
    }
}

fn scan_uninstall_registry(result: &mut ScanResult) {
    for key in [
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ] {
        let Ok(output) = reg_query(&[key, "/s"]) else {
            continue;
        };

        for entry in parse_uninstall_entries(&output) {
            let Some(name) = entry.name else {
                continue;
            };
            let Some(path) = entry.install_location else {
                continue;
            };
            if is_likely_game(&name, &path) {
                add_game(result, name, path, GameSource::WindowsUninstallRegistry);
            }
        }
    }
}

#[derive(Debug, Default)]
struct UninstallEntry {
    name: Option<String>,
    install_location: Option<PathBuf>,
}

fn parse_uninstall_entries(output: &str) -> Vec<UninstallEntry> {
    let mut entries = Vec::new();
    let mut current = UninstallEntry::default();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("HKEY_") {
            if current.name.is_some() || current.install_location.is_some() {
                entries.push(current);
            }
            current = UninstallEntry::default();
            continue;
        }

        let fields = trimmed.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 3 {
            continue;
        }
        match fields[0] {
            "DisplayName" => current.name = Some(fields[2..].join(" ")),
            "InstallLocation" => {
                let value = fields[2..].join(" ");
                let value = value.trim_matches('"');
                if !value.is_empty() {
                    current.install_location = Some(PathBuf::from(value));
                }
            }
            _ => {}
        }
    }

    if current.name.is_some() || current.install_location.is_some() {
        entries.push(current);
    }
    entries
}

fn is_likely_game(name: &str, path: &Path) -> bool {
    if !path.is_dir() || name.trim().is_empty() {
        return false;
    }

    let lower_name = name.to_ascii_lowercase();
    let lower_path = path.to_string_lossy().to_ascii_lowercase();
    let excluded = [
        "anticheat",
        "armoury",
        "audacity",
        "asus",
        "balena",
        "brave",
        "camera",
        "chrom",
        "cinny",
        "corsair",
        "discord",
        "docker",
        "ffmpeg",
        "figma",
        "framework service",
        "git",
        "gog galaxy",
        "google",
        "jdownloader",
        "microsoft",
        "mistral",
        "motiv mix",
        "musehub",
        "nexus mods",
        "node.js",
        "nord",
        "nvidia",
        "nvcpl",
        "occt",
        "ota dependencies",
        "plex",
        "powertoys",
        "qt",
        "realtek",
        "rizin",
        "rode",
        "redistributable",
        "sshpass",
        "system assistant",
        "teracopy",
        "tinymedia",
        "tmodloader",
        "treesize",
        "ubisoft connect",
        "visual c++",
        "visual studio",
        "virtual display",
        "wallpaper engine",
        "directx",
        "runtime",
        "uninstall",
        "anti-cheat",
        "launcher",
        "sdk",
        "vlc",
        "voice commands",
        "wifiman",
    ];
    let excluded_path_parts = ["\\asus\\", "\\microsoft\\", "\\nexusmods", "\\shure\\"];
    !excluded.iter().any(|part| lower_name.contains(part))
        && !excluded_path_parts
            .iter()
            .any(|part| lower_path.contains(part))
        && !lower_path.contains("nvidia")
}

fn is_likely_steam_game(name: &str) -> bool {
    let lower_name = name.to_ascii_lowercase();
    ![
        "steamworks common redistributables",
        "wallpaper engine",
        "occt",
        "tmodloader",
    ]
    .iter()
    .any(|part| lower_name == *part)
}

fn add_game(result: &mut ScanResult, name: String, path: PathBuf, source: GameSource) {
    let path = fs::canonicalize(&path).unwrap_or(path);
    let path = path
        .to_str()
        .and_then(|path| path.strip_prefix(r"\\?\"))
        .map(PathBuf::from)
        .unwrap_or(path);
    if path.is_dir() {
        result.games.push(Game { name, path, source });
    }
}

fn deduplicate_games(games: &mut Vec<Game>) {
    let mut seen = Vec::<String>::new();
    games.retain(|game| {
        let key = game.path.to_string_lossy().to_ascii_lowercase();
        if seen.iter().any(|existing| existing == &key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}

fn registry_value(key: &str, value_name: &str) -> Option<PathBuf> {
    let output = reg_query(&[key]).ok()?;
    let value = output.lines().find_map(|line| {
        let fields = line.trim().split_whitespace().collect::<Vec<_>>();
        (fields.len() >= 3 && fields[0] == value_name).then(|| fields[2..].join(" "))
    })?;
    (!value.is_empty()).then(|| PathBuf::from(unescape_vdf(&value)))
}

fn reg_query(arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("reg")
        .arg("query")
        .args(arguments)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn find_vdf_value(contents: &str, key: &str) -> Option<String> {
    contents
        .lines()
        .find_map(|line| parse_quoted_value(line, key))
}

fn parse_quoted_value(line: &str, key: &str) -> Option<String> {
    let values = quoted_values(line);
    (values.len() >= 2 && values[0] == key).then(|| values[1].clone())
}

fn quoted_values(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' && in_quotes {
            current.push(character);
            escaped = true;
        } else if character == '"' {
            if in_quotes {
                values.push(std::mem::take(&mut current));
            }
            in_quotes = !in_quotes;
        } else if in_quotes {
            current.push(character);
        }
    }
    values
}

fn unescape_vdf(value: &str) -> String {
    value.replace("\\\\", "\\")
}

fn parse_json_string(contents: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\"");
    let start = contents.find(&marker)? + marker.len();
    let remainder = contents[start..].trim_start();
    let remainder = remainder.strip_prefix(':')?.trim_start();
    let remainder = remainder.strip_prefix('"')?;

    let mut value = String::new();
    let mut escaped = false;
    for character in remainder.chars() {
        if escaped {
            value.push(match character {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some(value);
        } else {
            value.push(character);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_steam_values() {
        assert_eq!(
            parse_quoted_value(r#"    "path"    "C:\\\\Games\\Steam""#, "path"),
            Some(r#"C:\\\\Games\\Steam"#.to_string())
        );
        assert_eq!(
            find_vdf_value("\"name\"\t\"Example Game\"", "name"),
            Some("Example Game".to_string())
        );
    }

    #[test]
    fn parses_uninstall_registry_blocks() {
        let output = r#"
HKEY_LOCAL_MACHINE\Software\Example
    DisplayName    REG_SZ    Example Game
    InstallLocation    REG_SZ    C:\Games\Example
HKEY_LOCAL_MACHINE\Software\Runtime
    DisplayName    REG_SZ    Visual C++ Runtime
    InstallLocation    REG_SZ    C:\Runtime
"#;
        let entries = parse_uninstall_entries(output);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name.as_deref(), Some("Example Game"));
        assert_eq!(
            entries[0].install_location,
            Some(PathBuf::from(r"C:\Games\Example"))
        );
    }

    #[test]
    fn parses_epic_manifest_values() {
        let json = r#"{"DisplayName":"Example Game","InstallLocation":"C:\\Games\\Example"}"#;
        assert_eq!(
            parse_json_string(json, "DisplayName"),
            Some("Example Game".into())
        );
        assert_eq!(
            parse_json_string(json, "InstallLocation"),
            Some(r"C:\Games\Example".into())
        );
    }

    #[test]
    fn filters_known_non_game_registry_entries() {
        assert!(!is_likely_game(
            "NVIDIA App",
            Path::new(r"C:\Program Files\NVIDIA Corporation")
        ));
        assert!(!is_likely_game(
            "Wallpaper Engine",
            Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\wallpaper_engine")
        ));
        assert!(is_likely_game("Trackmania", Path::new(r"C:\Windows")));
    }
}
