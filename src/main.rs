use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use regex::Regex;
use walkdir::DirEntry;
use walkdir::WalkDir;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

// #[derive(Parser)]
// struct Args {
//     #[command(subcommand)]
//     command: Command,
// }

#[derive(Parser)]
enum Args {
    /// Computes the coverage of all project tags. 
    ComputeCoverage,

    /// Given a req or a beh will compute a hash print it. 
    GetHash { project_tag: String},
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
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

use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::io::{BufReader, Read, Write};

fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn hash_file<P: AsRef<Path>>(file: P) -> Result<Digest> {
    sha256_digest(std::fs::File::open(file)?)
}

fn get_project<P: AsRef<Path>>(root_path: P, tag: &str) -> Result<PathBuf> {
    for req in gather_files(root_path.as_ref().join("docs").join("reqs")).filter(|f|
                                                                        f.extension() == Some(OsStr::new("req"))) {
        if req.file_name() == Some(OsStr::new(tag)) {
            return Ok(req);
        }
    }
    for req in gather_files(root_path.as_ref().join("docs").join("behs")).filter(|f|
                                                                        f.extension() == Some(OsStr::new("beh"))) {
        if req.file_name() == Some(OsStr::new(tag)) {
            return Ok(req);
        }
    }
    Err(format!("Unable to find {}", tag).into())
}
 
fn get_all_project_entries<P: AsRef<Path>>(root_path:P) -> Result<HashMap<String, Digest>> {
    let mut results = HashMap::new();

    for (filepath, hash) in gather_files(root_path.as_ref().join("docs").join("reqs")).filter(|f|
                                                                                 f.extension() == Some(OsStr::new("req")))
    .map(|f| (f.clone(), hash_file(f).unwrap()))
    {
        results.insert(filepath.file_name().unwrap().to_str().unwrap().to_string(), hash);
    }
    Ok(results)
}

#[derive(Debug)]
struct Item {
    filepath: PathBuf,
    // line number. 
    // line: usize,
    tag: String,
    hash: String,
}

/// Scanns all files looking for project tag markers. 
/// rename to get all items. 
fn compute_coverage<P: AsRef<Path>>(root_dir: P) -> Result<Vec<Item>> {
    let regex = Regex::new(r"([0-9]+\.(beh|req))@SHA256:([a-f0-9]+)")?;

    let files = gather_files(root_dir);
    let mut items = vec![];
    for f in files {
        let content = std::fs::read_to_string(&f)?;
        for capture in regex.captures_iter(&content) {
            let i = Item {
                filepath: f.clone(),
                // line: capture.get(0)?.start(),
                // SAFTEY: can only get into function for each entry. 
                tag: capture.get(1).unwrap().as_str().to_string(),
                hash: capture.get(3).unwrap().as_str().to_string()
            };
            items.push(i);
        }
    }
    Ok(items)
}

fn main() -> Result<()> {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match Args::parse() {
        Args::GetHash { project_tag } => {
            let s = get_project(&root_dir, &project_tag)?;
            println!("Hash: {:?}", hash_file(s)?);
        },
        Args::ComputeCoverage => {
            let all_projects_items = get_all_project_entries(&root_dir)?;
            // todo: doesn't need to be a hash map
            let mut uncovered_items: HashMap<String, bool> = all_projects_items.iter().map(|(key, _)| (key.clone(), false)).collect();
            let items = compute_coverage(root_dir)?;
            
            for i in items {
                println!("I: {:?}", i);
                uncovered_items.remove(&i.tag);
            }
            println!("Uncovered items: \n{:?}", uncovered_items);
        }
    };
    Ok(())
}


#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn folder_with_contents() {
        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let f: Vec<_> = gather_files(root_dir.join("src")).collect();
        assert!(!f.is_empty());
    }

    // tests: 003.beh@SHA256:ffb10f4a84ebd6dfd13ff224af2f9d2ecf924d9e737e2fb952fce4bd76e0c39c
    #[test]
    fn invalid_folder() {
        let f = gather_files("non-existant folder");
        let k:Vec<_> = f.collect();
        assert!(k.is_empty());
    }

}
