use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use sha1_smol::Sha1;

#[derive(Debug)]
pub(crate) enum Object {
    Commit {
        path: PathBuf,
        parent_commit_digest: String,
        content: String,
        commit_msg: String,
        digest: String,
        author: String,
        timestamp: SystemTime,
    },
    Tree {
        path: PathBuf,
        content: String,
        digest: String,
    },
    Blob {
        path: PathBuf,
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

    // Calculates digest string for a content of a given object type. Which is a file content for
    // blob object, and formatted list of children objects
    pub fn update_digest(&mut self) {
        match self {
            // Self::Commit { path, .. } => _,
            // Self::Tree { path, .. } => path.as_ref(),
            Self::Blob {
                path,
                ref mut digest,
                ..
            } => {
                let content = fs::read_to_string(path.as_path()).unwrap();
                let mut hasher = Sha1::new();
                hasher.update(content.as_bytes());
                *digest = hasher.digest().to_string();
            }
            _ => panic!(),
        }
    }

    // It takes a formatted stirng representing a content of a child object, that contains an
    // object type, content digest and filename and concatenates this string to an object content.
    // The string will be used to calculate a parent digest. Function adds line break to the given
    // string.
    pub fn append_content(&mut self, obj_str: &str) {
        match self {
            Self::Commit {
                ref mut content, ..
            } => {
                content.push_str(obj_str);
                content.push_str("\n");
            }
            Self::Tree {
                ref mut content, ..
            } => {
                content.push_str(obj_str);
                content.push_str("\n");
            }
            Self::Blob { .. } => unreachable!(), // For blob content is a text file contains.
        }
    }

    // Format a string conains of an object type, its digest and a filename. Will panic if called
    // before digest is calculated.
    pub fn obj_content_line(&self) -> String {
        match self {
            Self::Commit { .. } => unreachable!(), // Commit can't be representet as an obj string.
            Self::Tree { path, digest, .. } => format!(
                "{}\t{}\t{}",
                super::TREE_DIR,
                digest.as_str(),
                path.file_name().unwrap().to_str().unwrap(),
            ),
            Self::Blob { path, digest, .. } => format!(
                "{}\t{}\t{}",
                super::BLOB_DIR,
                digest.as_str(),
                path.file_name().unwrap().to_str().unwrap(),
            ),
        }
    }
}
