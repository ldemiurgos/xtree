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
        if let EntryKind::SymLink { target } = &self.kind {
            if let Some(target) = target {
                write!(
                    f,
                    "{color}{style} {entry} -> {target} {style_reset}{color_reset}",
                    color = color::Fg(color::Cyan),
                    style = style::Italic,
                    color_reset = color::Fg(color::Reset),
                    style_reset = style::Reset,
                    entry = self.name,
                    target = target.display()
                )
            } else {
                write!(
                    f,
                    "{color}{style} {entry} -> x {style_reset}{color_reset}",
                    color = color::Fg(color::Cyan),
                    style = style::Italic,
                    color_reset = color::Fg(color::Reset),
                    style_reset = style::Reset,
                    entry = self.name
                )
            }
        } else if self.class == EntryClass::Directory {
            write!(
                f,
                "{color}{style}  {entry}{style_reset}{color_reset}",
                color = color::Fg(color::Blue),
                style = style::Bold,
                color_reset = color::Fg(color::Reset),
                style_reset = style::Reset,
                entry = self.name
            )
        } else {
            write!(f, "  {}", self.name)
        }
    }
}

struct Node {
    metadata: Entry,
    entries: Vec<Entry>,
}

impl Node {
    fn from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let metadata = Entry::from(&path)?;
        let mut entries: Vec<Entry> = fs::read_dir(&path)?
            .filter_map(|r| r.ok().map(|e| e.path()))
            .filter_map(|path| Entry::from(path).ok())
            .collect();
        entries.sort_by(Entry::cmp);
        Ok(Self { metadata, entries })
    }
}

struct Model {
    parent: Option<Node>,
    current: Node,
}

impl Model {
    fn initialize() -> io::Result<Self> {
        let current_path = env::current_dir()?;
        let parent_path = current_path.parent();
        let current = Node::from(&current_path)?;
        let parent = parent_path.and_then(|p| Node::from(p).ok());
        Ok(Self { parent, current })
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::new();
        if let Some(parent) = self.parent.as_ref() {
            buffer.push_str(format!("{}\n", parent.metadata).as_str());
            for (idx, parent_entry) in parent.entries.iter().enumerate() {
                buffer.push_str(
                    format!(
                        " {}{parent_entry}\n",
                        if idx == (parent.entries.len() - 1) {
                            "└──"
                        } else {
                            "├──"
                        }
                    )
                    .as_str(),
                );
                if parent_entry.name == self.current.metadata.name {
                    for (jdx, current_entry) in self.current.entries.iter().enumerate() {
                        buffer.push_str(if idx == (parent.entries.len() - 1) {
                            "   "
                        } else {
                            " │ "
                        });
                        buffer.push_str(
                            format!(
                                " {}{current_entry}\n",
                                if jdx == (self.current.entries.len() - 1) {
                                    "└──"
                                } else {
                                    "├──"
                                }
                            )
                            .as_str(),
                        );
                    }
                }
            }
        } else {
            buffer.push('/');
        }
        write!(f, "{buffer}")
    }
}

fn main() -> io::Result<()> {
    let model = Model::initialize()?;
    println!("{model}");
    Ok(())
}
