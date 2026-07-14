use std::io::{self, Write};

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    List,
    Add(String),
    Remove(String),
    Quit,
    Unknown(String),
}

#[derive(Debug, Default)]
struct ModHub {
    mods: Vec<String>,
}

impl ModHub {
    fn add_mod(&mut self, name: String) -> bool {
        if self.mods.iter().any(|existing| existing == &name) {
            return false;
        }

        self.mods.push(name);
        true
    }

    fn remove_mod(&mut self, name: &str) -> bool {
        if let Some(index) = self.mods.iter().position(|existing| existing == name) {
            self.mods.remove(index);
            true
        } else {
            false
        }
    }
}

fn parse_command(input: &str) -> Command {
    let mut parts = input.split_whitespace();
    let command = parts.next().unwrap_or_default().to_ascii_lowercase();
    let argument = parts.collect::<Vec<_>>().join(" ");

    match command.as_str() {
        "help" | "h" => Command::Help,
        "list" | "ls" => Command::List,
        "add" => Command::Add(argument),
        "remove" | "rm" => Command::Remove(argument),
        "quit" | "exit" | "q" => Command::Quit,
        _ => Command::Unknown(command),
    }
}

fn print_help() {
    println!("Befehle:");
    println!("  list                 Installierte Mods anzeigen");
    println!("  add <name>           Einen Mod hinzufügen");
    println!("  remove <name>        Einen Mod entfernen");
    println!("  help                 Diese Hilfe anzeigen");
    println!("  quit                 Anwendung beenden");
}

fn handle_command(hub: &mut ModHub, command: Command) -> bool {
    match command {
        Command::Help => print_help(),
        Command::List => {
            if hub.mods.is_empty() {
                println!("Noch keine Mods eingetragen.");
            } else {
                println!("Mods:");
                for (index, name) in hub.mods.iter().enumerate() {
                    println!("  {}. {}", index + 1, name);
                }
            }
        }
        Command::Add(name) if name.is_empty() => {
            println!("Bitte einen Mod-Namen angeben, z. B. `add example-mod`.");
        }
        Command::Add(name) => {
            if hub.add_mod(name.clone()) {
                println!("Mod hinzugefügt: {name}");
            } else {
                println!("Der Mod ist bereits vorhanden: {name}");
            }
        }
        Command::Remove(name) if name.is_empty() => {
            println!("Bitte einen Mod-Namen angeben, z. B. `remove example-mod`.");
        }
        Command::Remove(name) => {
            if hub.remove_mod(&name) {
                println!("Mod entfernt: {name}");
            } else {
                println!("Mod nicht gefunden: {name}");
            }
        }
        Command::Quit => {
            println!("Auf Wiedersehen!");
            return false;
        }
        Command::Unknown(command) => {
            println!("Unbekannter Befehl: {command}. `help` zeigt die verfügbaren Befehle.");
        }
    }

    true
}

fn run() -> io::Result<()> {
    let mut hub = ModHub::default();

    println!("ModHub – einfache Mod-Verwaltung");
    println!("`help` zeigt die verfügbaren Befehle.");

    loop {
        print!("modhub> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            println!();
            break;
        }

        if !handle_command(&mut hub, parse_command(&input)) {
            break;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands_and_arguments() {
        assert_eq!(
            parse_command("add cool mod"),
            Command::Add("cool mod".into())
        );
        assert_eq!(
            parse_command("RM cool mod"),
            Command::Remove("cool mod".into())
        );
        assert_eq!(parse_command("q"), Command::Quit);
    }

    #[test]
    fn adds_and_removes_mods_without_duplicates() {
        let mut hub = ModHub::default();

        assert!(hub.add_mod("example-mod".into()));
        assert!(!hub.add_mod("example-mod".into()));
        assert_eq!(hub.mods, vec!["example-mod"]);
        assert!(hub.remove_mod("example-mod"));
        assert!(!hub.remove_mod("example-mod"));
        assert!(hub.mods.is_empty());
    }
}
