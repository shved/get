mod object;
mod worktree;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::SystemTime;

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

pub fn init(cur_path: &mut PathBuf) {
    cur_path.push(REPO_DIR);

    if cur_path.as_path().is_dir() {
        panic!("get: repo already exist")
    }

    create_dirs(cur_path);

    create_files(cur_path);

    println!("Repo in `{}` created!", cur_path.display());
}

pub fn commit(cur_path: PathBuf, msg: Option<&str>) {
    if !cur_path.join(REPO_DIR).as_path().is_dir() {
        panic!("get: there is no repo in this directory, try to initialize it first")
    }

    // TODO Change default message to smthg more informative.
    let message = msg.unwrap_or("default commit message");

    worktree::commit(cur_path, message, IGNORE, SystemTime::now());
}

// TODO Unit test this (tree and permissions).
// TODO Remake to use std::fs::DirBuilder.
fn create_dirs(cur_path: &mut PathBuf) {
    // Crete `.get`.
    fs::create_dir(cur_path.as_path()).expect("get: can't create repo directory");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o755))
        .expect("get: can't set proper file ext permissions");

    // Crete `.get/objects`.
    create_dir(cur_path, OBJECTS_DIR);

    // Crete `.get/objects/*` dirs.
    cur_path.push(OBJECTS_DIR);
    create_dir(cur_path, COMMITS_DIR);
    create_dir(cur_path, TREE_DIR);
    create_dir(cur_path, BLOB_DIR);
    cur_path.pop();
}

fn create_dir(cur_path: &mut PathBuf, name: &str) {
    cur_path.push(name);
    fs::create_dir(cur_path.as_path()).expect("get: can't create commit directory.");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o755))
        .expect("get: can't set proper file ext permissions");
    cur_path.pop();
}

fn create_files(cur_path: &mut PathBuf) {
    cur_path.push(HEAD_FILE);
    fs::write(cur_path.as_path(), EMPTY_REF).expect("get: can't write head");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o644))
        .expect("get: can't set proper file ext permissions");
    cur_path.pop();

    cur_path.push(LOG_FILE);
    fs::File::create(cur_path.as_path()).expect("Unable to create LOG file");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o644))
        .expect("get: can't set proper file ext permissions");
    cur_path.pop();
}
