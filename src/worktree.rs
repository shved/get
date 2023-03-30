use crate::object::Object;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

type NodeId = usize;

struct Node {
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    id: NodeId,
}

struct Worktree {
    index: Vec<Object>,
    graph: Node,
}

impl Worktree {
    fn from_files(p: PathBuf, message: &str, ignore: &[&str], timestamp: SystemTime) -> Worktree {
        let mut wt = Worktree {
            index: Vec::new(),
            graph: Node {
                id: 0,
                parent: None,
                children: Vec::new(),
            },
        };

        let parent_commit_hash = read_head(p.as_path());

        let mut commit = Object::Commit {
            path: p.clone(),
            parent_commit_digest: parent_commit_hash,
            digest: String::default(),
            commit_msg: message.to_string(),
            timestamp,
        };

        wt.index.push(commit);

        traverse(&mut wt, 0, ignore);

        wt
    }
}

fn traverse(wt: &mut Worktree, cur_node: NodeId, ignore: &[&str]) {
    for entry in fs::read_dir(wt.index[cur_node].path())
        .expect("get: can't read dir")
        .into_iter()
        .filter(|e| !is_ignored(e.as_ref().unwrap().path(), ignore))
    {
        // We intentionally panic here to avoid silent errors with files access.
        let e = entry.unwrap();
        let ftype = e.file_type().unwrap();
        if ftype.is_dir() {
            // TODO
            // * make object
            // * push object to index
            // * make node with connection to parent
            // * add the node to parents children
        } else if ftype.is_file() {
        } else if ftype.is_symlink() {
        }
    }
}

pub(crate) fn commit(cur_path: PathBuf, message: &str, ignore: &[&str], timestamp: SystemTime) {
    let _ = Worktree::from_files(cur_path, message, ignore, timestamp);
}

fn is_ignored(e: PathBuf, ignored: &[&str]) -> bool {
    // TODO handle None from repo_root.parent()
    for i in ignored.iter() {
        if e.strip_prefix(e.parent().unwrap())
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

fn read_head(repo_root: &Path) -> String {
    fs::read_to_string(repo_root.join(super::REPO_DIR).join(super::HEAD_FILE))
        .expect("get: cant read HEAD")
        .trim()
        .into()
}
