use crate::error::Error;
use crate::object::Object;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use log::warn;
use users::get_current_username;

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
pub(crate) struct Worktree(Vec<Node>);

pub(crate) fn from_files(
    p: PathBuf,
    parent_commit_digest: String,
    message: &str,
    ignore: &[&str],
    now: SystemTime,
) -> Result<Worktree, Error> {
    let timestamp = now
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::Unexpected)?;

    // TODO Try to use .get.toml config and use current user id as a fallback.
    let author = get_current_username()
        .unwrap_or_else(|| {
            warn!("couldn't fetch user name, default user name used instead");
            OsString::from("unknown author")
        })
        .into_string()
        .map_err(|_| Error::Unexpected)?;

    let commit = Object::Commit {
        path: p,
        content: Vec::new(),
        properties: vec![
            parent_commit_digest,
            author,
            timestamp.as_secs().to_string(),
            message.to_string(),
        ],
        message: message.to_string(),
        timestamp,
        digest: String::default(),
    };

    let node = Node {
        children: Vec::new(),
        obj: commit,
    };

    let mut wt = Worktree(vec![node]);

    build_tree(&mut wt, 0, ignore)?;

    wt.0[0].obj.update_digest()?;

    Ok(wt)
}

impl Worktree {
    pub(crate) fn persist_commit(&self) -> Result<&str, Error> {
        let root_path = self.0[0].obj.path();

        save_all_children(root_path, self, 0)?;

        Ok(self.0[0].obj.digest())
    }
}

fn build_tree(wt: &mut Worktree, current: NodeId, ignore: &[&str]) -> Result<(), Error> {
    let mut new_cur: usize = Default::default();

    let entries = fs::read_dir(wt.0[current].obj.path())?;
    for entry in entries {
        let e = entry?;

        if is_ignored(e.path(), ignore)? {
            continue;
        }

        let ftype = e.file_type()?;
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

            build_tree(wt, new_cur, ignore)?;
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

            wt.0[new_cur].obj.update_digest()?;
        } else if ftype.is_symlink() {
            unimplemented!("get: we don't deal with symlinks here, please use real CVS like git")
        }

        // Append a parent object content with new child.
        let content_line = wt.0[new_cur].obj.obj_content_line()?;
        wt.0[current].obj.append_content(content_line);

        // Update current node digest.
        wt.0[current].obj.update_digest()?;
    }

    Ok(())
}

fn save_all_children(root_path: &Path, wt: &Worktree, cursor: usize) -> Result<(), Error> {
    wt.0[cursor].obj.save_object(root_path)?;

    for i in wt.0[cursor].children.as_slice() {
        save_all_children(root_path, wt, *i)?;
    }

    Ok(())
}

fn is_ignored(path: PathBuf, ignored: &[&str]) -> Result<bool, Error> {
    for i in ignored.iter() {
        let parent_path = path.parent().ok_or(Error::Unexpected)?;
        let relative_path = path
            .strip_prefix(parent_path)
            .map_err(|_| Error::Unexpected)?
            .to_str()
            .ok_or(Error::Unexpected)?;

        if relative_path.contains(i) {
            return Ok(true);
        }
    }

    Ok(false)
}
