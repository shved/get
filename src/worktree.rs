use crate::error::Error;
use crate::object::{Object, ObjectString};
use crate::paths;
use crate::Repo;
use crate::{DEFAULT_DIR_PERMISSIONS, DEFAULT_FILE_PERMISSIONS, DEFAULT_IGNORE};

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use itertools::Itertools;

type NodeId = usize;

#[derive(Debug, Clone)]
struct Node {
    children: Vec<NodeId>,
    obj: Object,
}

#[derive(Debug, Clone)]
/// Datastructure to hold all the file objects for a commit. Uses vector as a memory arena, but
/// elements are linked by the indexes used as pointers. It is very handy since we only need a tree
/// to build it, calculate digests and put it on the disk.
pub(crate) struct Worktree(Vec<Node>);

#[derive(Debug, Clone)]
pub(crate) struct RepoWithState {
    pub repo: Repo,
    pub wt: Worktree,
}

impl RepoWithState {
    pub(crate) fn from_files(
        repo: Repo,
        message: &str,
        now: SystemTime,
    ) -> Result<RepoWithState, Error> {
        let timestamp = now
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Unexpected)?;

        let author: &str = repo.config.author.as_ref();

        let commit = Object::Commit {
            path: repo.work_dir.clone(),
            content: Vec::new(),
            properties: vec![
                repo.head.clone(),
                author.to_string(),
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

        build_tree_from_files(&mut wt, 0, &repo)?;

        wt.0[0].obj.update_digest()?;

        Ok(RepoWithState { repo, wt })
    }

    pub(crate) fn from_commit(repo: Repo, digest: String) -> Result<RepoWithState, Error> {
        let commit = repo.read_commit_object(digest)?;

        if !matches!(commit, Object::Commit { .. }) {
            // TODO Do smthg with this crap.
            return Err(Error::Unexpected);
        }

        let node = Node {
            children: Vec::new(),
            obj: commit,
        };

        let mut wt = Worktree(vec![node]);

        wt.restore_tree_from_storage(&repo, 0)?;

        Ok(RepoWithState { repo, wt })
    }

    pub(crate) fn save_commit(&self) -> Result<&str, Error> {
        let working_dir = self.wt.0[0].obj.path();

        self.save_all_children(working_dir, 0)?;

        Ok(self.wt.0[0].obj.digest())
    }

    pub(crate) fn restore_files(self) -> Result<(), Error> {
        for node in self.wt.0.into_iter() {
            match node.obj {
                Object::Commit { .. } => (),
                Object::Tree { path, .. } => {
                    let path_to_restore = self.repo.work_dir.join(path);
                    fs::create_dir_all(&path_to_restore)?;
                    fs::set_permissions(
                        path_to_restore,
                        fs::Permissions::from_mode(DEFAULT_DIR_PERMISSIONS),
                    )?;
                }
                Object::Blob { path, content, .. } => {
                    let path_to_restore = self.repo.work_dir.join(path);
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

    fn save_all_children(&self, dir: &Path, cursor: usize) -> Result<(), Error> {
        self.repo.save_object(&self.wt.0[cursor].obj)?;

        for i in self.wt.0[cursor].children.as_slice() {
            self.save_all_children(dir, *i)?;
        }

        Ok(())
    }
}

impl Repo {
    fn build_children(&self, lines: Vec<String>, parent_path: PathBuf) -> Result<Vec<Node>, Error> {
        let mut res = Vec::<Node>::new();

        for l in lines {
            let parts: ObjectString = l
                .split("\t")
                .map(|s| s.to_string())
                .collect_tuple()
                .unwrap();

            let node = match parts.0.as_ref() {
                paths::TREE_DIR => {
                    let tree = self.from_tree(parts.1, parent_path.join(parts.2))?;

                    let node = Node {
                        children: Vec::new(),
                        obj: tree,
                    };

                    node
                }
                paths::BLOB_DIR => {
                    let blob = self.from_blob(parts.1, parent_path.join(parts.2))?;

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
}

impl Worktree {
    fn restore_tree_from_storage(&mut self, repo: &Repo, i: NodeId) -> Result<(), Error> {
        let mut children: Vec<Node>;

        match &self.0.get(i).ok_or(Error::Unexpected)?.obj {
            Object::Commit { content, path, .. } => {
                children = repo.build_children(content.clone(), path.clone())?;
            }
            Object::Tree { content, path, .. } => {
                children = repo.build_children(content.clone(), path.clone())?;
            }
            Object::Blob { .. } => {
                children = Vec::<Node>::new();
            }
        }

        if children.len() > 0 {
            self.0.append(&mut children);

            for ix in (i + 1)..self.0.len() {
                self.0
                    .get_mut(i)
                    .ok_or(Error::Unexpected)?
                    .children
                    .push(ix);
                self.restore_tree_from_storage(repo, ix)?;
            }
        }

        Ok(())
    }
}

pub(crate) fn clean_before_restore(p: &Path, repo: &Repo) -> Result<(), Error> {
    let entries = fs::read_dir(p)?.map(|e| e.unwrap());

    for e in entries {
        if is_ignored(&e.path(), &repo.config.ignore, DEFAULT_IGNORE) {
            continue;
        }

        let ftype = e.file_type()?;
        if ftype.is_dir() {
            clean_before_restore(&e.path(), repo)?;
            if fs::read_dir(&e.path()).into_iter().count() == 0 {
                fs::remove_dir(e.path())?;
            }
        } else if ftype.is_file() {
            fs::remove_file(e.path())?;
        } else if ftype.is_symlink() {
            unimplemented!("get: we don't deal with symlinks here, please use real CVS like git");
        }
    }

    Ok(())
}

fn build_tree_from_files(wt: &mut Worktree, current: NodeId, repo: &Repo) -> Result<(), Error> {
    let mut new_cur: usize = Default::default();

    let entries = fs::read_dir(repo.work_dir.join(wt.0[current].obj.path()))?;

    for entry in entries {
        let e = entry?;

        if is_ignored(&e.path(), &repo.config.ignore, DEFAULT_IGNORE) {
            continue;
        }

        let full_path = e.path();

        let relative_path = full_path
            .strip_prefix(repo.work_dir.as_path())
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

            build_tree_from_files(wt, new_cur, repo)?;
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

fn is_ignored(path: &PathBuf, ignored: &Vec<String>, default_ignored: &[&str]) -> bool {
    for pattern in ignored.iter() {
        for segment in path.components() {
            if segment == Component::Normal(pattern.as_ref()) {
                return true;
            }
        }
    }

    for pattern in default_ignored.iter() {
        for segment in path.components() {
            if segment == Component::Normal(pattern.as_ref()) {
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
        let ignore: Vec<String> = vec![
            ".git".to_string(),
            ".gitignore".to_string(),
            "target".to_string(),
            ".get".to_string(),
        ];

        const DEFAULT_IGNORE: &[&str] = &[".get", ".get.toml"]; // Default ignore patterns.

        let path = PathBuf::from("./hello/iamnot/ignore");
        assert!(!is_ignored(&path, &ignore, DEFAULT_IGNORE));

        let path = PathBuf::from("./edgecase/hello.get");
        assert!(!is_ignored(&path, &ignore, DEFAULT_IGNORE));

        let path = PathBuf::from("./oneanotheredgecase/mytarget/hey.rs");
        assert!(!is_ignored(&path, &ignore, DEFAULT_IGNORE));

        let path = PathBuf::from("./dir/target/hello");
        assert!(is_ignored(&path, &ignore, DEFAULT_IGNORE));

        let path = PathBuf::from("./dir/.git/hello");
        assert!(is_ignored(&path, &ignore, DEFAULT_IGNORE));
    }
}
