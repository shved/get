use crate::object::Object;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use users;

type NodeId = usize;

#[derive(Debug)]
struct Node {
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
            path: p,
            parent_commit_digest: parent_commit_hash,
            content: Vec::new(),
            digest: String::default(),
            commit_msg: message.to_string(),
            // TODO Try to use .get.toml config and use current user id as a fallback.
            author: users::get_current_username()
                .unwrap_or_else(|| OsString::from("unknown author"))
                .into_string()
                .unwrap(),
            timestamp,
        };

        let node = Node {
            children: Vec::new(),
            obj: commit,
        };

        let mut wt = Worktree(vec![node]);

        build_tree(&mut wt, 0, ignore);

        wt.0[0].obj.update_digest();

        wt
    }

    // fn persist_commit(&self) -> &str {
    //     persist_all_children(&self, 0);
    // self.0[0].obj.digest.as_str()
    // }
}

pub(crate) fn commit(cur_path: PathBuf, message: &str, ignore: &[&str], timestamp: SystemTime) {
    dbg!(Worktree::from_files(cur_path, message, ignore, timestamp));
    // let wt = Worktree::from_files(cur_path, message, ignore, timestamp);
    // wt.persist_commit();
    // TODO update_head();
}

fn build_tree(wt: &mut Worktree, current: NodeId, ignore: &[&str]) {
    let mut new_cur: usize = Default::default();

    for entry in fs::read_dir(wt.0[current].obj.path())
        .expect("get: can't read dir")
        .filter(|e| !is_ignored(e.as_ref().unwrap().path(), ignore))
    {
        let e = entry.expect("get: couldn't read fs entry"); // We fail explicitly here in case of issues with fs access.
        let ftype = e.file_type().unwrap();
        if ftype.is_dir() {
            let tree = Object::Tree {
                path: e.path(),
                content: Vec::new(),
                digest: String::default(),
            };

            let node = Node {
                children: Vec::new(),
                obj: tree,
            };

            wt.0.push(node); // Put new node in arena vector.
            new_cur = wt.0.len() - 1;

            wt.0[current].children.push(new_cur); // Update parent's children with new node.

            build_tree(wt, new_cur, ignore);
        } else if ftype.is_file() {
            let blob = Object::Blob {
                path: e.path(),
                content: String::default(),
                digest: String::default(),
            };

            let node = Node {
                children: Vec::new(),
                obj: blob,
            };

            wt.0.push(node); // Put new node in arena vector.
            new_cur = wt.0.len() - 1;

            wt.0[current].children.push(new_cur); // Update parent's children with new node.
                                                  //
            wt.0[new_cur].obj.update_digest();
        } else if ftype.is_symlink() {
            unimplemented!("get: we don't deal with symlinks here, please use real CVS like git")
        }

        // Append a parent object content with new child.
        let content_line = wt.0[new_cur].obj.obj_content_line();
        wt.0[current].obj.append_content(content_line);

        // Update current node digest.
        wt.0[current].obj.update_digest();
    }
}

fn persist_all_children(wt: &Worktree, cursor: usize) {
    wt.0[cursor].obj.persist_object();

    for index in wt.0[cursor].children.as_slice() {
        persist_all_children(wt, *index);
    }
}

fn is_ignored(e: PathBuf, ignored: &[&str]) -> bool {
    // TODO Handle None from repo_root.parent().
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
        .expect("get: can't read HEAD")
        .trim()
        .into()
}

fn write_head(repo_root: &Path, digest: &str) {
    fs::write(
        repo_root.join(super::REPO_DIR).join(super::HEAD_FILE),
        digest,
    )
    .expect("get: can't write HEAD");
}
