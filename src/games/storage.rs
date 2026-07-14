use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::{Game, GameSource};

const STORAGE_FILE: &str = "games.tsv";

/// Der Katalog liegt im aktuellen Arbeitsverzeichnis der Anwendung.
pub fn saved_games_path() -> PathBuf {
    PathBuf::from(STORAGE_FILE)
}

/// Speichert den aktuellen Scan und ersetzt dabei den bisherigen Katalog.
pub fn save_games(path: &Path, games: &[Game]) -> io::Result<()> {
    let mut contents = String::from("# ModHub-Spielekatalog v1\n");
    contents.push_str("name\tpath\tsource\n");

    for game in games {
        contents.push_str(&encode_field(&game.name));
        contents.push('\t');
        contents.push_str(&encode_field(&game.path.to_string_lossy()));
        contents.push('\t');
        contents.push_str(&game.source.to_string());
        contents.push('\n');
    }

    fs::write(path, contents)
}

pub fn load_saved_games(path: &Path) -> io::Result<Vec<Game>> {
    let contents = fs::read_to_string(path)?;
    let mut games = Vec::new();

    for line in contents.lines().skip(2) {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 {
            continue;
        }

        let Some(source) = parse_source(fields[2]) else {
            continue;
        };
        games.push(Game {
            name: decode_field(fields[0]),
            path: PathBuf::from(decode_field(fields[1])),
            source,
        });
    }

    Ok(games)
}

fn parse_source(value: &str) -> Option<GameSource> {
    match value {
        "Steam" => Some(GameSource::Steam),
        "Epic Games" => Some(GameSource::EpicGames),
        "Windows-Registrierung" => Some(GameSource::WindowsUninstallRegistry),
        _ => None,
    }
}

fn encode_field(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\t', "%09")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn decode_field(value: &str) -> String {
    value
        .replace("%09", "\t")
        .replace("%0D", "\r")
        .replace("%0A", "\n")
        .replace("%25", "%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_games_with_paths() {
        let path = std::env::temp_dir().join(format!("modhub-games-{}.tsv", std::process::id()));
        let games = vec![Game {
            name: "Example % Game".into(),
            path: PathBuf::from(r"C:\Games\Example"),
            source: GameSource::Steam,
        }];

        save_games(&path, &games).unwrap();
        let loaded = load_saved_games(&path).unwrap();
        let _ = fs::remove_file(path);

        assert_eq!(loaded, games);
    }
}
