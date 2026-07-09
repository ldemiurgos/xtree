use std::{
    cmp, env,
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};

use termion::{color, style};

#[derive(Debug)]
enum EntryKind {
    Regular,
    SymLink { target: Option<PathBuf> },
}

#[derive(Debug, PartialEq)]
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
            .unwrap_or("...".to_string());
        let path = path.as_ref().to_path_buf();
        let mut kind = EntryKind::Regular;
        let mut class = EntryClass::File;
        let mut metadata = fs::symlink_metadata(&path)?;

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

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.class == EntryClass::Directory {
            write!(
                f,
                "{color}{style}{entry}{style_reset}{color_reset}",
                color = color::Fg(color::Blue),
                style = style::Bold,
                color_reset = color::Fg(color::Reset),
                style_reset = style::Reset,
                entry = self.name
            )
        } else {
            write!(f, "{}", self.name)
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
}

fn main() -> io::Result<()> {
    let path = env::current_dir()?;
    let current_node = Node::from(path)?;
    for entry in current_node.entries.iter() {
        println!("{entry}");
    }
    Ok(())
}
