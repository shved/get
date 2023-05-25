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

pub(crate) fn repo_dir(cur_dir: &Path) -> Result<PathBuf, Error> {
    for a in cur_dir.ancestors() {
        if a.join(REPO_DIR).is_dir() {
            return Ok(a.to_owned());
        }
    }

    Err(Error::NotAGetRepo)
}

pub(crate) fn check_no_repo_dir(cur_dir: &PathBuf) -> Result<(), Error> {
    if cur_dir.join(REPO_DIR).is_dir() {
        return Err(Error::RepoAlreadyExist);
    }

    Ok(())
}

pub(crate) fn set_working_dir(cur_dir: &Path) -> Result<(), Error> {
    let working_dir = repo_dir(cur_dir)?;
    WORK_DIR.set(working_dir).ok();

    Ok(())
}

pub(crate) fn get_working_dir() -> Result<&'static PathBuf, Error> {
    let working_dir = WORK_DIR.get().ok_or(Error::WorkingDirNotSet)?;

    Ok(working_dir)
}
