use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::ZlibEncoder;
use flate2::Compression;
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

    pub fn digest(&self) -> &str {
        match self {
            Self::Commit { digest, .. } => digest.as_str(),
            Self::Tree { digest, .. } => digest.as_str(),
            Self::Blob { digest, .. } => digest.as_str(),
        }
    }

    // Calculates digest string for a content of a given object type. Which is a file content for
    // blob object, and formatted list of children objects for commit and tree node. It also sorts
    // objects content. Once digest is calculated content should'nt be altered.
    pub fn update_digest(&mut self) {
        match self {
            Self::Commit {
                content,
                parent_commit_digest,
                author,
                timestamp,
                commit_msg,
                digest,
                ..
            } => {
                let unix_time = timestamp.duration_since(UNIX_EPOCH).unwrap();
                let mut hasher = Sha1::new();
                // We sort because of possible difference in dir listing on different platforms.
                content.sort();
                // And then append all the other data valuable for commit.
                content.push(format!("{}\n", parent_commit_digest));
                content.push(format!("{}\n", author));
                content.push(format!("{}\n", unix_time.as_millis()));
                content.push(format!("{}\n", commit_msg));

                for line in content {
                    hasher.update(line.as_bytes());
                }

                *digest = hasher.digest().to_string();
            }
            Self::Tree {
                content, digest, ..
            } => {
                let mut hasher = Sha1::new();
                content.sort(); // We sort because of possible difference in dir listing on different platforms.
                for line in content {
                    hasher.update(line.as_bytes());
                }
                *digest = hasher.digest().to_string();
            }
            Self::Blob {
                path,
                digest,
                content,
                ..
            } => {
                let file_content = fs::read_to_string(path.as_path()).unwrap();
                let mut hasher = Sha1::new();
                hasher.update(file_content.as_bytes());
                *content = file_content;
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
                "{}\t{}\t{}\n",
                super::TREE_DIR,
                digest.as_str(),
                path.file_name().unwrap().to_str().unwrap(),
            ),
            Self::Blob { path, digest, .. } => format!(
                "{}\t{}\t{}\n",
                super::BLOB_DIR,
                digest.as_str(),
                path.file_name().unwrap().to_str().unwrap(),
            ),
        }
    }

    pub fn persist_object(&self, repo_root: &Path) {
        let mut zipper = ZlibEncoder::new(Vec::new(), Compression::default());

        match self {
            Self::Commit {
                content, digest, ..
            } => {
                zipper.write_all(content.join("").as_bytes()).unwrap();
                let compressed_bytes = zipper.finish().unwrap();
                write_gzip(
                    repo_root
                        .join(super::REPO_DIR)
                        .join(super::OBJECTS_DIR)
                        .join(super::COMMITS_DIR)
                        .as_path(), // TODO Do something with this crap.
                    digest,
                    compressed_bytes.as_ref(),
                )
            }
            Self::Tree {
                content, digest, ..
            } => {
                zipper.write_all(content.join("").as_bytes()).unwrap();
                let compressed_bytes = zipper.finish().unwrap();
                write_gzip(
                    repo_root
                        .join(super::REPO_DIR)
                        .join(super::OBJECTS_DIR)
                        .join(super::TREE_DIR)
                        .as_path(), // TODO Do something with this crap.
                    digest,
                    compressed_bytes.as_ref(),
                )
            }
            Self::Blob {
                content, digest, ..
            } => {
                zipper.write_all(content.as_bytes()).unwrap();
                let compressed_bytes = zipper.finish().unwrap();
                write_gzip(
                    repo_root
                        .join(super::REPO_DIR)
                        .join(super::OBJECTS_DIR)
                        .join(super::BLOB_DIR)
                        .as_path(), // TODO Do something with this crap.
                    digest,
                    compressed_bytes.as_ref(),
                )
            }
        }
    }

    pub fn persist_source_file(&self) {
        unimplemented!();
    }
}

fn write_gzip(path: &Path, digest: &String, data: &[u8]) {
    // I don't mind it will be false in case of permissions error. We are not doing rock solid
    // software here.
    if !path.join(digest).exists() {
        fs::write(path.join(digest).as_path(), data).expect("get: error writing an object");
    }
}
