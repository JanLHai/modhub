//! Plattformunabhängige Schnittstelle für die Erkennung installierter Spiele.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameSource {
    Steam,
    EpicGames,
    WindowsUninstallRegistry,
}

impl std::fmt::Display for GameSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = match self {
            Self::Steam => "Steam",
            Self::EpicGames => "Epic Games",
            Self::WindowsUninstallRegistry => "Windows-Registrierung",
        };

        formatter.write_str(source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub name: String,
    pub path: PathBuf,
    pub source: GameSource,
}

#[derive(Debug, Default)]
pub struct ScanResult {
    pub games: Vec<Game>,
    pub warnings: Vec<String>,
}

pub trait GameScanner {
    fn scan(&self) -> ScanResult;
}

#[cfg(windows)]
mod windows;

mod storage;

pub use storage::{load_saved_games, save_games, saved_games_path};

#[cfg(windows)]
pub use windows::WindowsGameScanner;

#[cfg(not(windows))]
mod unsupported;

#[cfg(not(windows))]
pub use unsupported::UnsupportedGameScanner;

pub fn scan_installed_games() -> ScanResult {
    #[cfg(windows)]
    {
        return WindowsGameScanner::default().scan();
    }

    #[cfg(not(windows))]
    {
        UnsupportedGameScanner.scan()
    }
}
