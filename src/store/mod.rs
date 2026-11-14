use crate::Notes;
use std::{fs, io, path::PathBuf};

fn path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gpui-notes")
        .join("notes.json")
}

fn example_note() -> Notes {
    Notes {
        id: 0,
        title: "Quick Example".to_string(),
        text: include_str!("../../readme.md").to_string(),
    }
}

pub fn load() -> Vec<Notes> {
    let notes = match fs::read(path()) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|e| {
            eprintln!("error loading notes: {e}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    };

    if notes.is_empty() {
        let notes = vec![example_note()];
        if let Err(e) = save(&notes) {
            eprintln!("error saving initial note: {e}");
        }
        notes
    } else {
        notes
    }
}

pub fn save(notes: &[Notes]) -> io::Result<()> {
    let path = path();
    fs::create_dir_all(path.parent().unwrap())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec(notes)?)?;
    fs::rename(tmp, path)
}
