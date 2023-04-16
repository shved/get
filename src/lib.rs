pub mod error;
mod object;
mod paths;
mod worktree;

use crate::error::Error;
use crate::paths::*;
use crate::worktree::Worktree;

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DEFAULT_FILE_PERMISSIONS: u32 = 0o644;
const DEFAULT_DIR_PERMISSIONS: u32 = 0o755;
const EMPTY_REF: &str = "0000000000000000000000000000000000000000";
const IGNORE: &[&str] = &[".git", ".gitignore", "target", ".get"];

pub fn init(cur_path: &mut PathBuf) -> Result<(), Error> {
    check_no_repo_dir(cur_path)?;

    create_utility_dirs(cur_path)?;
    create_utility_files(cur_path)?;

    Ok(())
}

pub fn commit(root_path: PathBuf, msg: Option<&str>, now: SystemTime) -> Result<String, Error> {
    check_repo_dir(&root_path)?;

    // TODO Change default message to smthg more informative.
    let message = msg.unwrap_or("default commit message");
    let parent_commit_digest = read_head(root_path.as_path())?;

    let wt = Worktree::from_files(
        root_path.clone(),
        parent_commit_digest,
        message,
        IGNORE,
        now,
    )?;

    let new_commit_digest = wt.save_commit().map(|s| s.to_string())?;

    write_head(root_path.as_path(), new_commit_digest.as_str())?;

    Ok(new_commit_digest)
}

pub fn restore(root_path: PathBuf, digest: &str) -> Result<(), Error> {
    check_repo_dir(&root_path)?;

    let wt = Worktree::from_commit(&root_path, digest.to_owned())?;

    wt.restore_files(root_path.clone())?;

    write_head(root_path.as_path(), digest)?;

    Ok(())
}

fn create_utility_dirs(cur_path: &mut PathBuf) -> Result<(), Error> {
    // Crete `.get`.
    cur_path.push(paths::REPO_DIR);
    create_dir(cur_path)?;

    // Crete `.get/objects`.
    cur_path.push(paths::OBJECTS_DIR);
    create_dir(cur_path)?;

    // Crete `.get/objects/*` dirs.
    cur_path.push(paths::COMMITS_DIR);
    create_dir(cur_path)?;
    cur_path.pop();

    cur_path.push(paths::TREE_DIR);
    create_dir(cur_path)?;
    cur_path.pop();

    cur_path.push(paths::BLOB_DIR);
    create_dir(cur_path)?;
    cur_path.pop();

    cur_path.pop();
    cur_path.pop();

    Ok(())
}

fn create_dir(cur_path: &mut PathBuf) -> io::Result<()> {
    fs::create_dir(cur_path.as_path())?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_DIR_PERMISSIONS),
    )?;

    Ok(())
}

fn create_utility_files(cur_path: &mut PathBuf) -> io::Result<()> {
    cur_path.push(paths::REPO_DIR);
    cur_path.push(paths::HEAD_FILE);
    fs::write(cur_path.as_path(), EMPTY_REF)?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_FILE_PERMISSIONS),
    )?;
    cur_path.pop();

    cur_path.push(paths::LOG_FILE);
    fs::File::create(cur_path.as_path())?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_FILE_PERMISSIONS),
    )?;
    cur_path.pop();

    cur_path.pop();

    Ok(())
}

fn read_head(repo_root: &Path) -> Result<String, Error> {
    let str = fs::read_to_string(paths::head_path(repo_root))?;

    Ok(str)
}

fn write_head(repo_root: &Path, digest: &str) -> Result<(), Error> {
    fs::write(paths::head_path(repo_root), digest)?;

    Ok(())
}
