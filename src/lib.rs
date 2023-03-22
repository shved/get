use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

// TODO change type to byte array
const EMPTY_REF: &str = "0000000000000000000000000000000000000000";

pub fn init(cur_path: &mut PathBuf) {
    cur_path.push(".get");

    let repo_initialized = cur_path.as_path().is_dir();
    if repo_initialized {
        panic!("Repo already exist.")
    }

    create_dirs(cur_path);

    create_files(cur_path);

    println!("Repo in `{}` created!", cur_path.display());
}

// TODO unit test this (tree and permissions)
// TODO remake to use std::fs::DirBuilder
fn create_dirs(cur_path: &mut PathBuf) {
    // Crete `.get`.
    fs::create_dir(cur_path.as_path()).expect("Unable to create repo directory.");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o755))
        .expect("Unable to set proper file ext permissions");

    // Crete `.get/objects`.
    create_dir(cur_path, "objects");

    // Crete `.get/objects/*` dirs.
    cur_path.push("objects");
    create_dir(cur_path, "commit");
    create_dir(cur_path, "tree");
    create_dir(cur_path, "blob");
    cur_path.pop();
}

fn create_dir(cur_path: &mut PathBuf, name: &str) {
    cur_path.push(name);
    fs::create_dir(cur_path.as_path()).expect("Unable to create commit directory.");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o755))
        .expect("Unable to set proper file ext permissions");
    cur_path.pop();
}

fn create_files(cur_path: &mut PathBuf) {
    cur_path.push("HEAD");
    fs::write(cur_path.as_path(), EMPTY_REF).expect("Unable to write head");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o644))
        .expect("Unable to set proper file ext permissions");
    cur_path.pop();

    cur_path.push("LOG");
    fs::File::create(cur_path.as_path()).expect("Unable to create LOG file");
    fs::set_permissions(cur_path.as_path(), fs::Permissions::from_mode(0o644))
        .expect("Unable to set proper file ext permissions");
    cur_path.pop();
}

pub fn commit(msg: Option<&String>) {
    match msg {
        Some(message) => println!("{}", message),
        None => println!("default commit message"), // TODO change it to smthg more sensible.
    }
}
