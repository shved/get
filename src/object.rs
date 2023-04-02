use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha1_smol::Sha1;

#[derive(Debug)]
pub(crate) enum Object {
    Commit {
        path: PathBuf,
        parent_commit_digest: String,
        content: Vec<String>,
        commit_msg: String,
        digest: String,
        author: String,
        timestamp: SystemTime,
    },
    Tree {
        path: PathBuf,
        content: Vec<String>,
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
            Self::Commit {
                content,
                ref mut digest,
                parent_commit_digest,
                author,
                commit_msg,
                timestamp,
                ..
            } => {
                let unix_time = timestamp.duration_since(UNIX_EPOCH).unwrap();
                let mut hasher = Sha1::new();
                hasher.update(content[0].as_bytes());
                hasher.update(parent_commit_digest.as_bytes());
                hasher.update(author.as_bytes());
                hasher.update(format!("{}", unix_time.as_millis()).as_bytes());
                hasher.update(commit_msg.as_bytes());
                *digest = hasher.digest().to_string();
            }
            Self::Tree {
                ref mut content,
                ref mut digest,
                ..
            } => {
                let mut hasher = Sha1::new();
                content.sort(); // We sort beacuaseo of possible difference in dir listing on different platforms.
                for line in content {
                    hasher.update(line.as_bytes());
                }
                *digest = hasher.digest().to_string();
            }
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
        }
    }

    // It takes a formatted stirng representing a content of a child object, that contains an
    // object type, content digest and filename and concatenates this string to an object content.
    // The string will be used to calculate a parent digest. Function adds line break to the given
    // string.
    pub fn append_content(&mut self, obj_str: String) {
        match self {
            Self::Commit {
                ref mut content, ..
            } => content.push(obj_str),
            Self::Tree {
                ref mut content, ..
            } => content.push(obj_str),
            Self::Blob { .. } => (), // For blob a content is what file contains.
        }
    }

    // Format a string conains of an object type, its digest and a filename. Will panic if called
    // before digest is calculated.
    pub fn obj_content_line(&self) -> String {
        match self {
            Self::Commit { .. } => String::new(), // Commit can't be representet as an obj string.
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
