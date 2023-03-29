use std::path::PathBuf;
use std::time::SystemTime;

pub(crate) enum Object {
    Commit {
        children: Vec<Object>,
        parent_commit_digest: String,
        commit_msg: String,
        commit_digest: String,
        timestamp: SystemTime,
        path: PathBuf,
    },
    Tree {
        parent: Option<Box<Object>>,
        children: Vec<Object>,
        path: PathBuf,
    },
    Blob {
        parent: Option<Box<Object>>,
        path: PathBuf,
        content: String,
    },
}
