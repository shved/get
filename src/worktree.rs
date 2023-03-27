use crate::object;
use std::env;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::{DirEntry, WalkDir};

struct Worktree {
    graph: Option<object::Object>,
}

impl Worktree {
    fn fs_snapshot(p: PathBuf, ignore: &[&str]) -> *mut Worktree {
        let repo_root = p.as_path();
        for entry in WalkDir::new(repo_root.to_str().unwrap())
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_ignored(e, repo_root, ignore))
            // Skip first entry (the root of repo).
            .skip(1)
        {
            dbg!(entry.unwrap());
        }

        &mut Worktree { graph: None }
    }
}

pub(crate) fn commit(_message: &str, ignore: &[&str], _timestamp: SystemTime) {
    let _ = Worktree::fs_snapshot(env::current_dir().unwrap(), ignore);
}

fn is_ignored(e: &DirEntry, repo_root: &Path, ignored: &[&str]) -> bool {
    // TODO handle None from repo_root.parent()
    for i in ignored.iter() {
        if e.path()
            .strip_prefix(repo_root.parent().unwrap())
            .unwrap()
            .to_str()
            .unwrap()
            .contains(i)
        {
            return true;
        }
    }

    false
}
