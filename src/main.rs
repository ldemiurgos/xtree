use std::{
    cmp, env, fs, io,
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
    fn from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let name = path
            .as_ref()
            .file_name()
            .map(|n| n.display().to_string())
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidFilename,
                "I was unable to retrieve the filename",
            ))?;
        let mut kind = EntryKind::Regular;
        let mut class = EntryClass::File;
        let mut metadata = fs::metadata(&path)?;

        if metadata.is_symlink() {
            let target = fs::canonicalize(&path).ok();
            if let Some(target_path) = target.as_ref() {
                metadata = fs::metadata(target_path)?;
            }
            kind = EntryKind::SymLink { target };
        }

        if metadata.is_dir() {
            class = EntryClass::Directory
        }

        Ok(Self { name, kind, class })
    }

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
        let mut entries: Vec<Entry> = fs::read_dir(&path)?
            .filter_map(|r| r.ok().map(|e| e.path()))
            .filter_map(|path| Entry::from(path).ok())
            .collect();
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
