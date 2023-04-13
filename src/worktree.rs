use crate::error::Error;
use crate::object::{Object, ObjectString};

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use itertools::Itertools;
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

impl Worktree {
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

        build_tree_from_files(&mut wt, 0, ignore)?;

        wt.0[0].obj.update_digest()?;

        Ok(wt)
    }

    pub(crate) fn from_commit(root_path: &PathBuf, digest: String) -> Result<Worktree, Error> {
        let commit = Object::read_commit(root_path, digest)?;

        if !matches!(commit, Object::Commit { .. }) {
            // TODO Do smthg with this crap.
            return Err(Error::Unexpected);
        }

        let node = Node {
            children: Vec::new(),
            obj: commit,
        };

        let mut wt = Worktree(vec![node]);

        restore_tree_from_storage(&root_path, &mut wt, 0)?;

        Ok(wt)
    }

    pub(crate) fn save_commit(&self) -> Result<&str, Error> {
        let root_path = self.0[0].obj.path();

        save_all_children(root_path, self, 0)?;

        Ok(self.0[0].obj.digest())
    }

    pub(crate) fn restore_files(&self) -> Result<(), Error> {
        Ok(())
    }
}

fn restore_tree_from_storage(
    root_path: &PathBuf,
    wt: &mut Worktree,
    i: NodeId,
) -> Result<(), Error> {
    let mut children: Vec<Node>;

    match &wt.0.get(i).ok_or(Error::Unexpected)?.obj {
        Object::Commit { content, path, .. } => {
            children = build_children(root_path, content.clone(), path.clone())?;
        }
        Object::Tree { content, path, .. } => {
            children = build_children(root_path, content.clone(), path.clone())?;
        }
        Object::Blob { .. } => {
            children = Vec::<Node>::new();
        }
    }

    if children.len() > 0 {
        wt.0.append(&mut children);

        for ix in i..wt.0.len() {
            wt.0.get_mut(i).ok_or(Error::Unexpected)?.children.push(ix);
            restore_tree_from_storage(root_path, wt, ix)?;
        }
    }

    Ok(())
}

fn build_children(
    root_path: &PathBuf,
    lines: Vec<String>,
    parent_path: PathBuf,
) -> Result<Vec<Node>, Error> {
    let mut res = Vec::<Node>::new();

    for l in lines {
        let parts: ObjectString = l
            .split("\t")
            .map(|s| s.to_string())
            .collect_tuple()
            .unwrap();

        let node = match parts.0.as_ref() {
            "tree" => {
                let tree = Object::read_tree(&root_path, parent_path.clone(), parts.1)?;

                let node = Node {
                    children: Vec::new(),
                    obj: tree,
                };

                node
            }
            "blob" => {
                let blob = Object::read_blob(&root_path, parent_path.clone(), parts.1)?;

                let node = Node {
                    children: Vec::new(),
                    obj: blob,
                };

                node
            }
            _ => unreachable!(),
        };

        res.push(node);
    }

    Ok(res)
}

fn build_tree_from_files(wt: &mut Worktree, current: NodeId, ignore: &[&str]) -> Result<(), Error> {
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

            build_tree_from_files(wt, new_cur, ignore)?;
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
