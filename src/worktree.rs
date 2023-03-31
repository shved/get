use crate::object::Object;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

type NodeId = usize;

#[derive(Debug)]
struct Node {
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    obj: Object,
}

#[derive(Debug)]
/// Datastructure to hold all the file objects for a commit. Uses vector as a memory arena, but
/// elements are linked by the indexes used as pointers. It is very handy since we only need a tree
/// to build it, calculate digests and put it on the disk.
struct Worktree(Vec<Node>);

impl Worktree {
    fn from_files(p: PathBuf, message: &str, ignore: &[&str], timestamp: SystemTime) -> Worktree {
        let parent_commit_hash = read_head(p.as_path());

        let commit = Object::Commit {
            path: p.clone(),
            parent_commit_digest: parent_commit_hash,
            digest: String::default(),
            commit_msg: message.to_string(),
            timestamp,
        };

        let node = Node {
            parent: None,
            children: Vec::new(),
            obj: commit,
        };

        let mut wt = Worktree(vec![node]);

        build_tree(&mut wt, 0, ignore);

        wt
    }
}

fn build_tree(wt: &mut Worktree, cur_i: NodeId, ignore: &[&str]) {
    for entry in fs::read_dir(wt.0[cur_i].obj.path())
        .expect("get: can't read dir")
        .into_iter()
        .filter(|e| !is_ignored(e.as_ref().unwrap().path(), ignore))
    {
        let e = entry.expect("get: couldn't read fs entry"); // We fail explicitly here in case of issues with fs access.
        let ftype = e.file_type().unwrap();
        if ftype.is_dir() {
            let tree = Object::Tree {
                path: e.path(),
                content: String::new(),
                digest: String::new(),
            };

            let node = Node {
                parent: Some(cur_i),
                children: Vec::new(),
                obj: tree,
            };

            wt.0.push(node); // Put new node in arena vector.
            let new_cur = wt.0.len() - 1;

            wt.0[cur_i].children.push(new_cur); // Update parent's children with new node.

            build_tree(wt, new_cur, ignore);
        } else if ftype.is_file() {
            let blob = Object::Blob {
                path: e.path(),
                content: String::new(),
                digest: String::new(),
            };

            let node = Node {
                parent: Some(cur_i),
                children: Vec::new(),
                obj: blob,
            };

            wt.0.push(node); // Put new node in arena vector.
            let new_cur = wt.0.len() - 1;

            wt.0[cur_i].children.push(new_cur); // Update parent's children with new node.
        } else if ftype.is_symlink() {
            panic!("get: we don't deal with symlinks here, please use real CVS like git")
        }
    }
}

pub(crate) fn commit(cur_path: PathBuf, message: &str, ignore: &[&str], timestamp: SystemTime) {
    dbg!(Worktree::from_files(cur_path, message, ignore, timestamp));
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
