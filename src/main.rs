use std::{
    cmp, env,
    fs::{self, canonicalize},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
enum EntryKind {
    Regular,
    SymLink { target: Option<PathBuf> },
}

#[derive(Debug)]
enum EntryClass {
    File,
    Directory,
}

#[derive(Debug)]
struct Entry {
    name: String,
    kind: EntryKind,
    class: EntryClass,
}

impl Entry {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        use EntryClass::*;
        match (&self.class, &other.class) {
            (File, File) | (Directory, Directory) => {
                self.name.to_lowercase().cmp(&other.name.to_lowercase())
            }
            (File, Directory) => cmp::Ordering::Greater,
            (Directory, File) => cmp::Ordering::Less,
        }
    }
}

struct Node {
    entries: Vec<Entry>,
}

impl Node {
    fn from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(&path)?.filter_map(|e| e.ok()) {
            let Some(name) = entry.file_name().into_string().ok() else {
                continue;
            };

            let metadata = fs::symlink_metadata(entry.path())?;
            let kind = if metadata.is_symlink() {
                let target = canonicalize(&path).ok();
                EntryKind::SymLink { target }
            } else {
                EntryKind::Regular
            };
            let class = match &kind {
                EntryKind::SymLink { target } => {
                    if target
                        .clone()
                        .and_then(|target| fs::metadata(target).ok())
                        .map(|m| m.is_dir())
                        .unwrap_or(false)
                    {
                        EntryClass::Directory
                    } else {
                        EntryClass::File
                    }
                }

                EntryKind::Regular => {
                    if metadata.is_dir() {
                        EntryClass::Directory
                    } else {
                        EntryClass::File
                    }
                }
            };

            let entry = Entry { name, kind, class };
            println!("{entry:?}");
            entries.push(entry);
        }

        entries.sort_by(Entry::cmp);
        Ok(Self { entries })
    }

    fn list_names_from_entries(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.name.clone()).collect()
    }
}

fn main() -> io::Result<()> {
    let path = env::current_dir()?;
    let current_node = Node::from(path)?;
    for filename in current_node.list_names_from_entries() {
        println!("{filename}");
    }
    Ok(())
}
