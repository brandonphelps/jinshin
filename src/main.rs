use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use walkdir::WalkDir;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ListNodes { project_root: String },
}

/// returns an iterator over all files within a directory
/// paths are absolute paths. 
fn gather_files<P: AsRef<Path>>(project_root: P) -> impl Iterator<Item=PathBuf> {
    WalkDir::new(project_root).into_iter()
        .filter_map(|f| {
            f.ok().and_then(|f| if f.file_type().is_file(){
                Some(f)
            } else {
                None
            }).map(|f| f.into_path())
    })
}

fn main() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let requirement_files = gather_files(root_dir.join("docs").join("reqs"))
        .filter(|f| {
            println!("Extenion: {:?}", f.extension());
            f.extension() == Some(OsStr::new("req"))
        });

    let behaviour_files = gather_files(root_dir.join("docs").join("behs"))
        .filter(|f| {
            f.extension() == Some(OsStr::new("beh"))
        });

    for file in requirement_files {
        println!("File: {:?}", file);
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;

    // tests: 003.beh
    #[test]
    fn invalid_folder() {
        let f = gather_files("non-existant folder");
        let k:Vec<_> = f.collect();
        assert!(k.is_empty());
    }

}
