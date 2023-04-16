use crate::error::Error;

use std::path::{Path, PathBuf};

pub(crate) const REPO_DIR: &str = ".get";
pub(crate) const HEAD_FILE: &str = "HEAD";
pub(crate) const LOG_FILE: &str = "LOG";
pub(crate) const OBJECTS_DIR: &str = "objects";
pub(crate) const COMMITS_DIR: &str = "commit";
pub(crate) const TREE_DIR: &str = "tree";
pub(crate) const BLOB_DIR: &str = "blob";

pub(crate) fn commits_path(repo_root: &Path) -> PathBuf {
    repo_root.join(REPO_DIR).join(OBJECTS_DIR).join(COMMITS_DIR)
}

pub(crate) fn tree_path(repo_root: &Path) -> PathBuf {
    repo_root.join(REPO_DIR).join(OBJECTS_DIR).join(TREE_DIR)
}

pub(crate) fn blob_path(repo_root: &Path) -> PathBuf {
    repo_root.join(REPO_DIR).join(OBJECTS_DIR).join(BLOB_DIR)
}

pub(crate) fn head_path(repo_root: &Path) -> PathBuf {
    repo_root.join(REPO_DIR).join(HEAD_FILE)
}

pub(crate) fn log_path(repo_root: &Path) -> PathBuf {
    repo_root.join(REPO_DIR).join(LOG_FILE)
}

pub(crate) fn check_repo_dir(repo_root: &PathBuf) -> Result<(), Error> {
    if repo_root.join(REPO_DIR).is_dir() {
        return Ok(());
    }

    Err(Error::NotAGetRepo)
}

pub(crate) fn check_no_repo_dir(repo_root: &PathBuf) -> Result<(), Error> {
    if repo_root.join(REPO_DIR).is_dir() {
        return Err(Error::RepoAlreadyExist);
    }

    Ok(())
}
