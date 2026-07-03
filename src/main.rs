use std::{env, fs, io};

fn main() -> io::Result<()> {
    let path = env::current_dir()?;
    let entries = fs::read_dir(&path)?;
    for entry in entries {
        let entry = entry?;
        if let Some(entry_name) = entry.path().file_name() {
            println!("{}", entry_name.display());
        }
    }
    Ok(())
}
