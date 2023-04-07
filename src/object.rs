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
        // Dir listing to save and calculate digest.
        content: Vec<String>,
        digest: String,
    },
    Blob {
        path: PathBuf,
        // Simply file content.
        content: String,
        digest: String,
    },
}

impl Object {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Commit { path, .. } => path.as_ref(),
            Self::Tree { path, .. } => path.as_ref(),
            Self::Blob { path, .. } => path.as_ref(),
        }
    }

    pub(crate) fn digest(&self) -> &str {
        match self {
            Self::Commit { digest, .. } => digest.as_str(),
            Self::Tree { digest, .. } => digest.as_str(),
            Self::Blob { digest, .. } => digest.as_str(),
        }
    }

    // Calculates digest string for a content of a given object type. Which is a file content for
    // blob object, and formatted list of children objects for commit and tree node. It also sorts
    // objects content. Once digest is calculated content should'nt be altered.
    pub(crate) fn update_digest(&mut self) -> Result<(), Error> {
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
            Self::Blob { path, digest, .. } => {
                let file_content = fs::read_to_string(path.as_path())?;
                let mut hasher = Sha1::new();
                hasher.update(file_content.as_bytes());
                *digest = hasher.digest().to_string();
            }
        }

        Ok(())
    }

    // It takes a formatted stirng representing a content of a child object, that contains an
    // object type, content digest and filename and concatenates this string to an object content.
    // The string will be used to calculate a parent digest. Function adds line break to the given
    // string.
    pub(crate) fn append_content(&mut self, obj_str: String) {
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
    pub(crate) fn obj_content_line(&self) -> Result<String, Error> {
        match self {
            Self::Commit { .. } => Ok(String::default()), // Commit can't be representet as an obj string.
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

    pub(crate) fn save_object(&self, repo_root: &Path) -> Result<(), Error> {
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
    pub(crate) fn restore_object(&self) -> Result<(), Error> {
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

#[cfg(test)]
mod test {
    use super::Object;
    use std::path::PathBuf;

    #[test]
    fn update_digest_commit() {
        let commit_properties = vec![
            String::from("1234567890abcdefghij"),
            String::from("rakhmaninov"),
            String::from("1680873704237"),
            String::from("descriptive commit message\nwith several\nlines."),
        ];

        let mut commit = Object::Commit {
            path: PathBuf::from("/tmp"),
            content: vec![
                String::from("blob\t32bab984c61ba43ba15e479b23df5e828aa43864\tCargo.lock\n"),
                String::from("blob\t77c57febfc94ff583a1a15a004d01cf6e16a4442\tCargo.toml\n"),
                String::from("blob\t9c00c0236a2b133dbfbc7ad799f9ea5ce685c2e4\tTODO.md\n"),
                String::from("blob\te28de939637c3a53b10c53ebfc3203acbf168717\t.rustfmt.toml\n"),
                String::from("tree\t17d520fea68d0d107a4e8becad26e47f37e73aab\tsrc\n"),
                String::from("tree\t9dffa2d73d8b2f67a59768600023cd32b21ba7ac\tdummy_app\n"),
            ],
            properties: commit_properties.clone(),
            digest: String::default(),
        };

        assert!(commit.update_digest().is_ok());
        assert!(commit.digest() == "36c07fe4f2d7e8b84510ea9f189be3cc575ee977");

        let mut commit_with_content_reordered = Object::Commit {
            path: PathBuf::from("/tmp"),
            content: vec![
                String::from("blob\te28de939637c3a53b10c53ebfc3203acbf168717\t.rustfmt.toml\n"),
                String::from("blob\t9c00c0236a2b133dbfbc7ad799f9ea5ce685c2e4\tTODO.md\n"),
                String::from("blob\t77c57febfc94ff583a1a15a004d01cf6e16a4442\tCargo.toml\n"),
                String::from("tree\t17d520fea68d0d107a4e8becad26e47f37e73aab\tsrc\n"),
                String::from("blob\t32bab984c61ba43ba15e479b23df5e828aa43864\tCargo.lock\n"),
                String::from("tree\t9dffa2d73d8b2f67a59768600023cd32b21ba7ac\tdummy_app\n"),
            ],
            properties: commit_properties.clone(),
            digest: String::default(),
        };

        assert!(commit_with_content_reordered.update_digest().is_ok());
        assert!(commit.digest() == commit_with_content_reordered.digest());
    }

    #[test]
    fn update_digest_tree() {
        let mut tree = Object::Tree {
            path: PathBuf::from("/tmp"),
            content: vec![
                String::from("blob\t0a883d942f72a18558810edd255d846f408ed35a\tmain.rs\n"),
                String::from("blob\t44dd4de05dddc1235fdb19bf1ab2dc4c11178da8\tobject.rs\n"),
                String::from("blob\t5651a0070d5d6031b7b9e53e7d962acfb9fdfba1\tworktree.rs\n"),
                String::from("blob\td4c47993f35fec5888d41e74aa67cda00242376a\tlib.rs\n"),
                String::from("blob\tf8cf364dd0b44a6c44c99600ff6d2ca9111c3a23\terror.rs\n"),
            ],
            digest: String::default(),
        };

        assert!(tree.update_digest().is_ok());
        assert!(tree.digest() == "2956f8a9a34d9b58b93d16426ee4689f5b2b7964");

        let mut tree_with_content_reordered = Object::Tree {
            path: PathBuf::from("/tmp"),
            content: vec![
                String::from("blob\td4c47993f35fec5888d41e74aa67cda00242376a\tlib.rs\n"),
                String::from("blob\t44dd4de05dddc1235fdb19bf1ab2dc4c11178da8\tobject.rs\n"),
                String::from("blob\t5651a0070d5d6031b7b9e53e7d962acfb9fdfba1\tworktree.rs\n"),
                String::from("blob\tf8cf364dd0b44a6c44c99600ff6d2ca9111c3a23\terror.rs\n"),
                String::from("blob\t0a883d942f72a18558810edd255d846f408ed35a\tmain.rs\n"),
            ],
            digest: String::default(),
        };

        assert!(tree_with_content_reordered.update_digest().is_ok());
        assert!(tree.digest() == tree_with_content_reordered.digest());
    }

    #[test]
    fn tree_obj_content_line() {
        let tree = Object::Tree {
            path: PathBuf::from("/tmp"),
            content: Vec::new(),
            digest: String::from("digest"),
        };

        let content_line = tree.obj_content_line();
        assert!(content_line.is_ok());
        assert!(content_line.unwrap() == String::from("tree\tdigest\ttmp\n"));
    }

    #[test]
    fn blob_obj_content_line() {
        let blob = Object::Blob {
            path: PathBuf::from("/tmp/odyssey.txt"),
            content: String::default(),
            digest: String::from("digest"),
        };

        let content_line = blob.obj_content_line();
        assert!(content_line.is_ok());
        assert!(content_line.unwrap() == String::from("blob\tdigest\todyssey.txt\n"));
    }
}
