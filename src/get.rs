use std::fs::create_dir;
use std::path::PathBuf;

pub(crate) fn init(cur_path: &mut PathBuf) {
    cur_path.push(".get");

    let as_path = cur_path.as_path();
    let repo_initialized = as_path.is_dir();
    if repo_initialized {
        panic!("Repo already exist.")
    }

    create_dir(as_path).expect("Unable to create repo directory.");

    cur_path.push("commit");
    create_dir(cur_path.as_path()).expect("Unable to create commit directory.");
    cur_path.pop();
}

pub(crate) fn commit(msg: Option<&String>) {
    match msg {
        Some(message) => println!("{}", message),
        None => println!("default commit message"), // TODO change it to smthg more senvible.
    }
}
