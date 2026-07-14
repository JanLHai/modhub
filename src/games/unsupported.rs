use super::{GameScanner, ScanResult};

/// Platzhalter für Systeme, für die noch kein Scanner implementiert ist.
/// Neue Plattformen können später implementiert werden, ohne die CLI zu ändern.
pub struct UnsupportedGameScanner;

impl GameScanner for UnsupportedGameScanner {
    fn scan(&self) -> ScanResult {
        ScanResult {
            games: Vec::new(),
            warnings: vec![
                "Die automatische Spieleerkennung ist derzeit nur für Windows verfügbar."
                    .to_string(),
            ],
        }
    }
}
