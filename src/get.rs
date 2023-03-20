use std::path::PathBuf;

pub(crate) fn init(_cur_path: PathBuf) {
    println!("Repo initialized.");
}

pub(crate) fn commit(msg: Option<&String>) {
            match msg {
                Some(message) => println!("{}", message),
                None => println!("default commit message"),
            }
}
