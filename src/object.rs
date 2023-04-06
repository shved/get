use crate::error::Error;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1_smol::Sha1;

#[derive(Debug)]
pub(crate) enum Object {
    Commit {
        path: PathBuf,
        // Repo root content to save and calculate digest.
        content: Vec<String>,
        // Additional properties for a commit to save and calculate digest.
        properties: Vec<String>,
        digest: String,
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
    pub fn update_digest(&mut self) -> Result<(), Error> {
        match self {
            Self::Commit {
                content,
                properties,
                digest,
                ..
            } => {
                let mut hasher = Sha1::new();
                // We sort because of possible difference in dir listing on different platforms.
                content.sort();
                for line in content {
                    hasher.update(line.as_bytes());
                }
                for line in properties {
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
                let file_content = fs::read_to_string(path.as_path())?;
                let mut hasher = Sha1::new();
                hasher.update(file_content.as_bytes());
                *content = file_content;
                *digest = hasher.digest().to_string();
            }
        }

        Ok(())
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
    pub fn obj_content_line(&self) -> Result<String, Error> {
        match self {
            Self::Commit { .. } => Ok(String::new()), // Commit can't be representet as an obj string.
            Self::Tree { path, digest, .. } => {
                let file_name = path
                    .file_name()
                    .ok_or(Error::Unexpected)?
                    .to_str()
                    .ok_or(Error::Unexpected)?;
                return Ok(format!(
                    "{}\t{}\t{}\n",
                    super::TREE_DIR,
                    digest.as_str(),
                    file_name
                ));
            }
            Self::Blob { path, digest, .. } => {
                let file_name = path
                    .file_name()
                    .ok_or(Error::Unexpected)?
                    .to_str()
                    .ok_or(Error::Unexpected)?;
                return Ok(format!(
                    "{}\t{}\t{}\n",
                    super::BLOB_DIR,
                    digest.as_str(),
                    file_name,
                ));
            }
        }
    }

    pub fn save_object(&self, repo_root: &Path) -> Result<(), Error> {
        let mut zipper = ZlibEncoder::new(Vec::new(), Compression::default());

        match self {
            Self::Commit {
                content,
                properties,
                digest,
                ..
            } => {
                zipper.write_all(content.join("").as_bytes())?;
                zipper.write_all(properties.join("").as_bytes())?;
                let compressed_bytes = zipper.finish()?;
                write_gzip(
                    repo_root // TODO Do something with this crap.
                        .join(super::REPO_DIR)
                        .join(super::OBJECTS_DIR)
                        .join(super::COMMITS_DIR)
                        .as_path(),
                    digest,
                    compressed_bytes.as_ref(),
                )?;
            }
            Self::Tree {
                content, digest, ..
            } => {
                zipper.write_all(content.join("").as_bytes())?;
                let compressed_bytes = zipper.finish()?;
                write_gzip(
                    repo_root // TODO Do something with this crap.
                        .join(super::REPO_DIR)
                        .join(super::OBJECTS_DIR)
                        .join(super::TREE_DIR)
                        .as_path(),
                    digest,
                    compressed_bytes.as_ref(),
                )?;
            }
            Self::Blob {
                content, digest, ..
            } => {
                zipper.write_all(content.as_bytes())?;
                let compressed_bytes = zipper.finish()?;
                write_gzip(
                    repo_root // TODO Do something with this crap.
                        .join(super::REPO_DIR)
                        .join(super::OBJECTS_DIR)
                        .join(super::BLOB_DIR)
                        .as_path(),
                    digest,
                    compressed_bytes.as_ref(),
                )?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn restore_object(&self) -> Result<(), Error> {
        unimplemented!();
    }
}

fn write_gzip(path: &Path, digest: &String, data: &[u8]) -> Result<(), Error> {
    // I don't mind it will be false in case of permissions error. We are not covering all the
    // real world edge cases here.
    if !path.join(digest).exists() {
        fs::write(path.join(digest).as_path(), data)?;
    }

    Ok(())
}
