// "2c92b1926814fe12defcc8859070354c8386e3d9"
pub mod error;
mod object;
mod paths;
mod worktree;

use crate::error::Error;
use crate::worktree::Worktree;

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::SystemTime;

const DEFAULT_FILE_PERMISSIONS: u32 = 0o644;
const DEFAULT_DIR_PERMISSIONS: u32 = 0o755;
const EMPTY_REF: &str = "0000000000000000000000000000000000000000";
const IGNORE: &[&str] = &[".git", ".gitignore", "target", ".get", ".get.toml"];

pub fn init(work_dir: &mut PathBuf) -> Result<(), Error> {
    paths::check_no_repo_dir(work_dir)?;

    create_utility_dirs(work_dir)?;
    create_utility_files(work_dir)?;

    Ok(())
}

pub fn commit(cur_dir: PathBuf, msg: Option<&str>, now: SystemTime) -> Result<String, Error> {
    let working_dir = paths::repo_dir(cur_dir.as_path())?;
    paths::set_working_dir(working_dir);

    // TODO Change default message to smthg more informative.
    let message = msg.unwrap_or("default commit message");
    let parent_commit_digest = read_head()?;

    let wt = Worktree::from_files(parent_commit_digest, message, now)?;

    let new_commit_digest = wt.save_commit().map(|s| s.to_string())?;

    write_head(new_commit_digest.as_str())?;

    Ok(new_commit_digest)
}

pub fn restore(cur_dir: PathBuf, digest: &str) -> Result<(), Error> {
    let working_dir = paths::repo_dir(cur_dir.as_path())?;
    paths::set_working_dir(working_dir);

    let wt = Worktree::from_commit(digest.to_owned())?;

    worktree::clean_before_restore()?;

    wt.restore_files()?;

    write_head(digest)?;

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

fn read_head() -> Result<String, Error> {
    let str = fs::read_to_string(paths::head_path())?;

    Ok(str)
}

fn write_head(digest: &str) -> Result<(), Error> {
    fs::write(paths::head_path(), digest)?;

    Ok(())
}
