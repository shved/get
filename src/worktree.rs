use crate::error::Error;
use crate::object::{Object, ObjectString};
use crate::paths;
use crate::IGNORE;
use crate::{DEFAULT_DIR_PERMISSIONS, DEFAULT_FILE_PERMISSIONS};

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
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
        parent_commit_digest: String,
        message: &str,
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
            path: paths::get_working_dir().unwrap().to_owned(),
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

        build_tree_from_files(&mut wt, 0)?;

        wt.0[0].obj.update_digest()?;

        Ok(wt)
    }

    pub(crate) fn from_commit(digest: String) -> Result<Worktree, Error> {
        let commit = Object::read_commit(digest)?;

        if !matches!(commit, Object::Commit { .. }) {
            // TODO Do smthg with this crap.
            return Err(Error::Unexpected);
        }

        let node = Node {
            children: Vec::new(),
            obj: commit,
        };

        let mut wt = Worktree(vec![node]);

        restore_tree_from_storage(&mut wt, 0)?;

        Ok(wt)
    }

    pub(crate) fn save_commit(&self) -> Result<&str, Error> {
        let working_dir = self.0[0].obj.path();

        save_all_children(working_dir, self, 0)?;

        Ok(self.0[0].obj.digest())
    }

    pub(crate) fn restore_files(self) -> Result<(), Error> {
        for node in self.0 {
            match node.obj {
                Object::Commit { .. } => (),
                Object::Tree { path, .. } => {
                    let path_to_restore = paths::get_working_dir().unwrap().join(path);
                    fs::create_dir_all(&path_to_restore)?;
                    fs::set_permissions(
                        path_to_restore,
                        fs::Permissions::from_mode(DEFAULT_DIR_PERMISSIONS),
                    )?;
                }
                Object::Blob { path, content, .. } => {
                    let path_to_restore = paths::get_working_dir().unwrap().join(path);
                    fs::write(&path_to_restore, content)?;
                    fs::set_permissions(
                        path_to_restore,
                        fs::Permissions::from_mode(DEFAULT_FILE_PERMISSIONS),
                    )?;
                }
            }
        }

        Ok(())
    }
}

pub(crate) fn clean_before_restore() -> Result<(), Error> {
    let entries = fs::read_dir(paths::get_working_dir().unwrap())?;
    for entry in entries {
        let e = entry?;

        if is_ignored(&e.path(), IGNORE) {
            continue;
        }

        let ftype = e.file_type()?;
        if ftype.is_dir() {
            fs::remove_dir_all(e.path())?;
        } else if ftype.is_file() {
            fs::remove_file(e.path())?;
        } else if ftype.is_symlink() {
            unimplemented!("get: we don't deal with symlinks here, please use real CVS like git")
        }
    }

    Ok(())
}

fn restore_tree_from_storage(wt: &mut Worktree, i: NodeId) -> Result<(), Error> {
    let mut children: Vec<Node>;

    match &wt.0.get(i).ok_or(Error::Unexpected)?.obj {
        Object::Commit { content, path, .. } => {
            children = build_children(content.clone(), path.clone())?;
        }
        Object::Tree { content, path, .. } => {
            children = build_children(content.clone(), path.clone())?;
        }
        Object::Blob { .. } => {
            children = Vec::<Node>::new();
        }
    }

    if children.len() > 0 {
        wt.0.append(&mut children);

        for ix in (i + 1)..wt.0.len() {
            wt.0.get_mut(i).ok_or(Error::Unexpected)?.children.push(ix);
            restore_tree_from_storage(wt, ix)?;
        }
    }

    Ok(())
}

fn build_children(lines: Vec<String>, parent_path: PathBuf) -> Result<Vec<Node>, Error> {
    let mut res = Vec::<Node>::new();

    for l in lines {
        let parts: ObjectString = l
            .split("\t")
            .map(|s| s.to_string())
            .collect_tuple()
            .unwrap();

        let node = match parts.0.as_ref() {
            paths::TREE_DIR => {
                let tree = Object::read_tree(parent_path.clone(), parts.1)?;

                let node = Node {
                    children: Vec::new(),
                    obj: tree,
                };

                node
            }
            paths::BLOB_DIR => {
                let blob = Object::read_blob(parent_path.clone(), parts.1)?;

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

fn build_tree_from_files(wt: &mut Worktree, current: NodeId) -> Result<(), Error> {
    let mut new_cur: usize = Default::default();

    let entries = fs::read_dir(
        paths::get_working_dir()
            .unwrap()
            .join(wt.0[current].obj.path()),
    )?;

    for entry in entries {
        let e = entry?;

        if is_ignored(&e.path(), IGNORE) {
            continue;
        }

        let full_path = e.path();

        let relative_path = full_path
            .strip_prefix(paths::get_working_dir().unwrap())
            .map_err(|_| Error::Unexpected)?;

        let ftype = e.file_type()?;
        if ftype.is_dir() {
            let tree = Object::Tree {
                path: relative_path.to_owned(),
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

            build_tree_from_files(wt, new_cur)?;
        } else if ftype.is_file() {
            let blob = Object::Blob {
                path: relative_path.to_owned(),
                full_path,
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
        if !content_line.is_empty() {
            wt.0[current].obj.append_content(content_line);
            // Update current node digest.
            wt.0[current].obj.update_digest()?;
        }
    }

    Ok(())
}

fn save_all_children(working_dir: &Path, wt: &Worktree, cursor: usize) -> Result<(), Error> {
    wt.0[cursor].obj.save_object()?;

    for i in wt.0[cursor].children.as_slice() {
        save_all_children(working_dir, wt, *i)?;
    }

    Ok(())
}

fn is_ignored(path: &PathBuf, ignored: &[&str]) -> bool {
    for pattern in ignored.iter() {
        for segment in path.components() {
            if segment.as_os_str() == *pattern {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ignored() {
        const IGNORE: &[&str] = &[".git", ".gitignore", "target", ".get"];

        let path = PathBuf::from("./hello/iamnot/ignore");
        assert!(!is_ignored(&path, IGNORE));

        let path = PathBuf::from("./edgecase/hello.get");
        assert!(!is_ignored(&path, IGNORE));

        let path = PathBuf::from("./oneanotheredgecase/mytarget/hey.rs");
        assert!(!is_ignored(&path, IGNORE));

        let path = PathBuf::from("./dir/target/hello");
        assert!(is_ignored(&path, IGNORE));

        let path = PathBuf::from("./dir/.git/hello");
        assert!(is_ignored(&path, IGNORE));
    }
}
