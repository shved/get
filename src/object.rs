use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub(crate) enum Object {
    Commit {
        path: PathBuf,
        parent_commit_digest: String,
        commit_msg: String,
        digest: String,
        timestamp: SystemTime,
    },
    Tree {
        path: PathBuf,
        content: String,
        digest: String,
    },
    Blob {
        path: PathBuf,
        content: String,
        digest: String,
    },
}

impl Object {
    pub fn path(&self) -> &Path {
        match self {
            Self::Commit { path, .. } => path.as_ref(),
            Self::Tree { path, .. } => path.as_ref(),
            Self::Blob { path, .. } => path.as_ref(),
        }
    }
}
