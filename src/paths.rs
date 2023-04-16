use crate::error::Error;

use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;

pub(crate) const REPO_DIR: &str = ".get";
pub(crate) const HEAD_FILE: &str = "HEAD";
pub(crate) const LOG_FILE: &str = "LOG";
pub(crate) const OBJECTS_DIR: &str = "objects";
pub(crate) const COMMITS_DIR: &str = "commit";
pub(crate) const TREE_DIR: &str = "tree";
pub(crate) const BLOB_DIR: &str = "blob";

pub(crate) static WORK_DIR: OnceCell<PathBuf> = OnceCell::new();

pub(crate) fn commits_path() -> PathBuf {
    get_working_dir()
        .unwrap()
        .join(REPO_DIR)
        .join(OBJECTS_DIR)
        .join(COMMITS_DIR)
}

pub(crate) fn tree_path() -> PathBuf {
    get_working_dir()
        .unwrap()
        .join(REPO_DIR)
        .join(OBJECTS_DIR)
        .join(TREE_DIR)
}

pub(crate) fn blob_path() -> PathBuf {
    get_working_dir()
        .unwrap()
        .join(REPO_DIR)
        .join(OBJECTS_DIR)
        .join(BLOB_DIR)
}

pub(crate) fn head_path() -> PathBuf {
    get_working_dir().unwrap().join(REPO_DIR).join(HEAD_FILE)
}

pub(crate) fn log_path() -> PathBuf {
    get_working_dir().unwrap().join(REPO_DIR).join(LOG_FILE)
}

pub(crate) fn check_repo_dir(working_dir: &Path) -> Result<(), Error> {
    if working_dir.join(REPO_DIR).is_dir() {
        return Ok(());
    }

    Err(Error::NotAGetRepo)
}

pub(crate) fn check_no_repo_dir(working_dir: &PathBuf) -> Result<(), Error> {
    if working_dir.join(REPO_DIR).is_dir() {
        return Err(Error::RepoAlreadyExist);
    }

    Ok(())
}

pub(crate) fn set_working_dir(working_dir: PathBuf) {
    // TODO here check the parents directories if the give directory doesnt contain a repo.
    WORK_DIR.set(working_dir).ok();
}

pub(crate) fn get_working_dir() -> Result<&'static PathBuf, Error> {
    let working_dir = WORK_DIR.get().ok_or(Error::WorkingDirNotSet)?;

    Ok(working_dir)
}
