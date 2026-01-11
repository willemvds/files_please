use std::fs;
use std::os::unix::fs::MetadataExt;
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
    pub inode: u64,
}

impl Entry {
    pub fn new(kind: EntryKind, name: path::PathBuf, inode: u64) -> Entry {
        Entry {
            kind: kind,
            name: name,
            inode: inode,
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
                            if let Ok(metadata) = entry.metadata() {
                                entries.entries.push(Entry::new(
                                    EntryKind::Dir,
                                    path::PathBuf::from(entry_name),
                                    metadata.ino(),
                                ));
                            }
                        }
                    } else if file_type.is_file() {
                        if let Some(entry_name) = entry.path().file_name() {
                            if let Ok(metadata) = entry.metadata() {
                                entries.entries.push(Entry::new(
                                    EntryKind::File,
                                    path::PathBuf::from(entry_name),
                                    metadata.ino(),
                                ));
                            }
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
