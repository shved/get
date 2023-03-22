use crate::object;
use std::env;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

struct Worktree {
    #[allow(dead_code)]
    graph: Option<object::Object>,
}

impl Worktree {
    fn fs_snapshot(p: PathBuf) -> *mut Worktree {
        for entry in WalkDir::new(p.as_path().to_str().unwrap()).follow_links(false) {
            println!("{}", entry.unwrap().path().display())
        }

        &mut Worktree { graph: None }
    }
}

// TODO fix it printing the whole /home structure
pub(crate) fn commit(_message: &str, _timestamp: SystemTime) {
    // println!("{}", env::current_dir().unwrap().display());
    // panic!("asdf");
    let _ = Worktree::fs_snapshot(env::current_dir().unwrap());
}
