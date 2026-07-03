use std::{
    env,
    fs::{self, DirEntry},
    io,
    path::Path,
};

struct Node {
    entries: Vec<DirEntry>,
}

impl Node {
    fn from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Self {
            entries: fs::read_dir(path)?.filter_map(|entry| entry.ok()).collect(),
        })
    }

    fn list_names_from_entries(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(|f| f.file_name().into_string().ok())
            .collect()
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
