use std::fs;
use std::path;

#[derive(Clone, PartialEq)]
pub enum EntryKind {
    Dir,
    File,
}

#[derive(Clone)]
pub struct Entry {
    pub kind: EntryKind,
    pub name: path::PathBuf,
}

impl Entry {
    pub fn new(kind: EntryKind, name: path::PathBuf) -> Entry {
        Entry {
            kind: kind,
            name: name,
        }
    }
}

#[derive(Clone)]
pub struct Entries {
    pub absolute_path: path::PathBuf,
    pub entries: Vec<Entry>,
}

impl Entries {
    pub fn new(absolute_path: path::PathBuf, read_dir_it: fs::ReadDir) -> Entries {
        let mut entries = Entries {
            absolute_path: absolute_path,
            entries: vec![],
        };

        for entry in read_dir_it {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(entry_name) = entry.path().file_name() {
                            entries
                                .entries
                                .push(Entry::new(EntryKind::Dir, path::PathBuf::from(entry_name)));
                        }
                    } else if file_type.is_file() {
                        if let Some(entry_name) = entry.path().file_name() {
                            entries
                                .entries
                                .push(Entry::new(EntryKind::File, path::PathBuf::from(entry_name)));
                        }
                    } else {
                        if let Some(entry_name) = entry.path().file_name() {
                            eprintln!(
                                "Unhandled file type {}",
                                path::PathBuf::from(entry_name).display()
                            );
                        }
                    }
                }
            }
        }

        entries
    }
}
