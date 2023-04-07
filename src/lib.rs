pub mod error;
mod object;
mod worktree;

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::error::Error;

const DEFAULT_FILE_PERMISSIONS: u32 = 0o644;
const DEFAULT_DIR_PERMISSIONS: u32 = 0o755;
const EMPTY_REF: &str = "0000000000000000000000000000000000000000";
const REPO_DIR: &str = ".get";
const HEAD_FILE: &str = "HEAD";
const LOG_FILE: &str = "LOG";
const OBJECTS_DIR: &str = "objects";
const COMMITS_DIR: &str = "commit";
const TREE_DIR: &str = "tree";
const BLOB_DIR: &str = "blob";
const IGNORE: &[&str] = &[
    ".git",
    ".gitignore",
    "target",
    ".get",
    "ignoredfolder",
    ".get.toml",
];

pub fn init(cur_path: &mut PathBuf) -> Result<(), Error> {
    cur_path.push(REPO_DIR);

    if cur_path.as_path().is_dir() {
        return Err(Error::RepoAlreadyExist);
    }

    create_dirs(cur_path)?;

    create_files(cur_path)?;

    cur_path.pop();

    Ok(())
}

pub fn commit(root_path: PathBuf, msg: Option<&str>, now: SystemTime) -> Result<String, Error> {
    if !root_path.join(REPO_DIR).as_path().is_dir() {
        return Err(Error::RepoAlreadyExist);
    }

    // TODO Change default message to smthg more informative.
    let message = msg.unwrap_or("default commit message");

    worktree::commit(root_path, message, IGNORE, now)
}

fn create_dirs(cur_path: &mut PathBuf) -> Result<(), Error> {
    // Crete `.get`.
    fs::create_dir(cur_path.as_path())?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_DIR_PERMISSIONS),
    )?;

    // Crete `.get/objects`.
    create_dir(cur_path, OBJECTS_DIR)?;

    // Crete `.get/objects/*` dirs.
    cur_path.push(OBJECTS_DIR);
    create_dir(cur_path, COMMITS_DIR)?;
    create_dir(cur_path, TREE_DIR)?;
    create_dir(cur_path, BLOB_DIR)?;
    cur_path.pop();

    Ok(())
}

fn create_dir(cur_path: &mut PathBuf, name: &str) -> io::Result<()> {
    cur_path.push(name);
    fs::create_dir(cur_path.as_path())?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_DIR_PERMISSIONS),
    )?;
    cur_path.pop();

    Ok(())
}

fn create_files(cur_path: &mut PathBuf) -> io::Result<()> {
    cur_path.push(HEAD_FILE);
    fs::write(cur_path.as_path(), EMPTY_REF)?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_FILE_PERMISSIONS),
    )?;
    cur_path.pop();

    cur_path.push(LOG_FILE);
    fs::File::create(cur_path.as_path())?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_FILE_PERMISSIONS),
    )?;
    cur_path.pop();

    Ok(())
}
