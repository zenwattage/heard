use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Write, Read};
use serde::{Serialize, Deserialize};
use dirs::home_dir;
use colored::*; // For text styling

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    text: String,
    category: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = setup_notes_file();

    if args.len() < 2 {
        eprintln!("{}", "Usage: heard [options]".red().bold());
        eprintln!("Options:");
        eprintln!("  heard \"note text\" : category      Add a new note");
        eprintln!("  heard --list [category]           List notes");
        eprintln!("  heard --edit INDEX \"new text\" : category   Edit a note");
        eprintln!("  heard --remove or --r INDEX              Remove a note");
        return;
    }

    match args[1].as_str() {
        "--list" | "-l" => {
            let category = if args.len() > 2 { Some(&args[2]) } else { None };
            list_notes(&file_path, category);
        }
        "--edit" => {
            println!("{}", file_path);
            if args.len() < 5 || args[3] != ":" {
                eprintln!("{}", "Usage: heard --edit INDEX \"new text\" : category".yellow());
                return;
            }
            let index: usize = args[2].parse().unwrap_or_else(|_| {
                eprintln!("{}", "Invalid index for edit.".red());
                std::process::exit(1);
            });
            let new_text = args[4].trim_matches('"').to_string();
            let new_category = args[5].to_string();
            edit_note(&file_path, index - 1, &new_text, &new_category);
        }
        "--remove" | "--r" => {
            if args.len() < 3 {
                eprintln!("{}", "Usage: heard --remove INDEX".yellow());
                return;
            }
            let index: usize = args[2].parse().unwrap_or_else(|_| {
                eprintln!("{}", "Invalid index for removal.".red());
                std::process::exit(1);
            });
            remove_note(&file_path, index - 1);
        }
        _ => {
            if args.len() < 4 || args[2] != ":" {
                eprintln!("{}", "Usage: heard \"note text\" : category".yellow());
                return;
            }
            let note_text = args[1].trim_matches('"').to_string();
            let category = args[3].to_string();
            let note = Note {
                text: note_text,
                category,
            };
            let mut notes = read_notes(&file_path);
            notes.push(note);
            if save_notes(&file_path, &notes).is_err() {
                eprintln!("{}", "Failed to save note.".red());
            } else {
                println!("{}", "Note saved successfully!".green());
            }
        }
    }
}

fn setup_notes_file() -> String {
    let notes_dir = home_dir()
        .map(|home| home.join(".heard"))
        .expect("Could not determine home directory");
    fs::create_dir_all(&notes_dir).expect("Failed to create notes directory");
    notes_dir.join("notes.json").to_string_lossy().to_string()
}

fn list_notes(file_path: &str, category: Option<&String>) {
    let notes = read_notes(file_path);

    if notes.is_empty() {
        println!("{}", "No notes found.".red().bold());
        return;
    }

    let filtered_notes: Vec<&Note> = match category {
        Some(cat) => notes.iter().filter(|note| &note.category == cat).collect(),
        None => notes.iter().collect(),
    };

    if filtered_notes.is_empty() {
        println!("{}", format!("No notes found for the category: {}", category.unwrap_or(&"any".to_string())).yellow());
    } else {
        println!("{}", "Notes:".green().bold());
        for (i, note) in filtered_notes.iter().enumerate() {
            let index = format!("{:>2}.", i + 1).cyan().bold();
            let category = format!("[{}]", note.category).blue();
            let icon = get_icon_for_category(&note.category);
            let text = format!("{}", note.text).white();

            println!("{} {} {} {}", index, icon, category, text);
        }
    }
}

fn edit_note(file_path: &str, index: usize, new_text: &str, new_category: &str) {
    let mut notes = read_notes(file_path);
    
    if index >= notes.len() {
        eprintln!("{}", "Invalid index. Note does not exist.".red());
        return;
    }

    notes[index].text = new_text.to_string();
    notes[index].category = new_category.to_string();

    if save_notes(file_path, &notes).is_err() {
        eprintln!("{}", "Failed to update note.".red());
    } else {
        println!("{}", "Note updated successfully!".green());
    }
}

fn remove_note(file_path: &str, index: usize) {
    let mut notes = read_notes(file_path);

    if index >= notes.len() {
        eprintln!("{}", "Invalid index. Note does not exist.".red());
        return;
    }

    notes.remove(index);

    if save_notes(file_path, &notes).is_err() {
        eprintln!("{}", "Failed to remove note.".red());
    } else {
        println!("{}", "Note removed successfully!".green());
    }
}

fn read_notes(file_path: &str) -> Vec<Note> {
    let mut notes = Vec::new();
    if let Ok(mut file) = OpenOptions::new().read(true).open(file_path) {
        let mut contents = String::new();
        if let Ok(_) = file.read_to_string(&mut contents) {
            if let Ok(parsed_notes) = serde_json::from_str::<Vec<Note>>(&contents) {
                notes = parsed_notes;
            }
        }
    }
    notes
}

fn save_notes(file_path: &str, notes: &[Note]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)?;
    let contents = serde_json::to_string_pretty(&notes).unwrap();
    write!(file, "{}", contents)
    

}

fn get_icon_for_category(category: &str) -> &'static str {
    match category {
        "shopping" => "🛍️",
        "work" => "💼",
        "personal" => "🌟",
        "study" => "📚",
        _ => "📝",
    }
}

