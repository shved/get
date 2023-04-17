use crate::error::Error;
use crate::paths;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use flate2::{read::GzDecoder, Compression, GzBuilder};
use sha1_smol::Sha1;

#[derive(Debug)]
pub(crate) enum Object {
    Commit {
        path: PathBuf,
        // Working directory content to save and calculate digest.
        content: Vec<String>,
        // Additional properties for a commit to save and calculate digest.
        properties: Vec<String>,
        message: String,
        timestamp: Duration,
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
        full_path: PathBuf,
        // Simply file content.
        content: String,
        digest: String,
    },
}

pub(crate) type ObjectString = (String, String, String); // Object type, digest and filename.

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
            Self::Blob {
                full_path,
                content,
                digest,
                ..
            } => {
                let file_content = fs::read_to_string(full_path.as_path())?;
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
                Ok(format!(
                    "{}\t{}\t{}",
                    paths::TREE_DIR,
                    digest.as_str(),
                    file_name
                ))
            }
            Self::Blob { path, digest, .. } => {
                let file_name = path
                    .file_name()
                    .ok_or(Error::Unexpected)?
                    .to_str()
                    .ok_or(Error::Unexpected)?;
                Ok(format!(
                    "{}\t{}\t{}",
                    paths::BLOB_DIR,
                    digest.as_str(),
                    file_name,
                ))
            }
        }
    }

    pub(crate) fn save_object(&self) -> Result<(), Error> {
        match self {
            Self::Commit {
                content,
                properties,
                message,
                timestamp,
                digest,
                ..
            } => {
                let f = File::create(paths::commits_path().join(digest))?;

                let mut zipper = GzBuilder::new()
                    .filename(digest.as_bytes())
                    .comment(message.as_bytes())
                    .extra(timestamp.as_secs().to_string().as_bytes())
                    .write(f, Compression::default());

                zipper.write_all(format_commit_properties(properties.clone()).as_bytes())?;
                zipper.write_all(content.join("\n").as_bytes())?;
                zipper.finish()?;
            }
            Self::Tree {
                path,
                content,
                digest,
                ..
            } => {
                let f = File::create(paths::tree_path().join(digest))?;

                let mut zipper = GzBuilder::new()
                    .filename(path.file_name().unwrap().to_str().unwrap())
                    // TODO save dir timestamp to restore it as well
                    // .extra(timestamp.as_secs().into())
                    .write(f, Compression::default());

                zipper.write_all(content.join("\n").as_bytes())?;
                zipper.finish()?;
            }
            Self::Blob {
                path,
                content,
                digest,
                ..
            } => {
                let f = File::create(paths::blob_path().join(digest))?;

                let mut zipper = GzBuilder::new()
                    .filename(path.file_name().unwrap().to_str().unwrap())
                    // TODO save file timestamp to restore it as well
                    // .extra(timestamp.as_secs().into())
                    .write(f, Compression::default());

                zipper.write_all(content.as_bytes())?;
                zipper.finish()?;
            }
        }

        Ok(())
    }

    pub(crate) fn read_commit(digest: String) -> Result<Object, Error> {
        let (_name, contents) =
            decode_archive(paths::commits_path().join(digest.clone()).as_path())?;

        let lines: Vec<String> = contents.split("\n").map(|s| s.to_owned()).collect();

        // Verify a commit has at least it's basic properties.
        if lines.len() < 4 {
            return Err(Error::Unexpected);
        }

        let commit = Object::Commit {
            path: paths::get_working_dir().unwrap().to_owned(),
            properties: lines[0..=3].to_vec(),
            content: lines[4..].to_vec(),
            message: lines[3].clone(),
            timestamp: Duration::new(lines[2].parse::<u64>().unwrap(), 0),
            digest,
        };

        Ok(commit)
    }

    pub(crate) fn read_tree(parent_path: PathBuf, digest: String) -> Result<Object, Error> {
        let (name, contents) = decode_archive(paths::tree_path().join(digest.clone()).as_path())?;

        let lines: Vec<String> = contents.split("\n").map(|s| s.to_owned()).collect();

        if lines.len() < 1 {
            return Err(Error::Unexpected);
        }

        let path = parent_path.join(name);

        let children: Vec<String> = contents.split("\n").map(|s| s.to_owned()).collect();

        let tree = Object::Tree {
            path,
            content: children,
            digest,
        };

        Ok(tree)
    }

    pub(crate) fn read_blob(parent_path: PathBuf, digest: String) -> Result<Object, Error> {
        let (name, content) = decode_archive(paths::blob_path().join(digest.clone()).as_path())?;

        let path = parent_path.join(name);

        let blob = Object::Blob {
            path,
            full_path: PathBuf::default(), // Not needed at this point.
            content,
            digest,
        };

        Ok(blob)
    }
}

// TODO Test it.
fn format_commit_properties(props: Vec<String>) -> String {
    let mut joined = props.join("\n");
    joined.push('\n');
    joined
}

fn decode_archive(path: &Path) -> Result<(String, String), Error> {
    let f = File::open(path)?;
    let mut decoder = GzDecoder::new(f);
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;
    let header = decoder.header().ok_or(Error::Unexpected)?;
    let filename_slice = header.filename().ok_or(Error::Unexpected)?;
    let filename_str =
        String::from_utf8(filename_slice.to_owned()).map_err(|_| Error::UnsupportedEncoding)?;

    Ok((filename_str, contents))
}

#[cfg(test)]
mod tests {
    use super::Object;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn update_digest_commit() {
        let message = "descriptive commit message with several lines.";
        let commit_properties = vec![
            String::from("1234567890abcdefghij"), // digest
            String::from("rakhmaninov"),          // author
            String::from("1680961369"),           // timestamp
            String::from(message),
        ];

        let mut commit = Object::Commit {
            path: PathBuf::from("/tmp"),
            content: vec![
                String::from("blob\t32bab984c61ba43ba15e479b23df5e828aa43864\tCargo.lock"),
                String::from("blob\t77c57febfc94ff583a1a15a004d01cf6e16a4442\tCargo.toml"),
                String::from("blob\t9c00c0236a2b133dbfbc7ad799f9ea5ce685c2e4\tTODO.md"),
                String::from("blob\te28de939637c3a53b10c53ebfc3203acbf168717\t.rustfmt.toml"),
                String::from("tree\t17d520fea68d0d107a4e8becad26e47f37e73aab\tsrc"),
                String::from("tree\t9dffa2d73d8b2f67a59768600023cd32b21ba7ac\tdummy_app"),
            ],
            properties: commit_properties.clone(),
            message: String::from(message),
            timestamp: Duration::new(1680961369, 0),
            digest: String::default(),
        };

        assert!(commit.update_digest().is_ok());
        assert!(commit.digest() == "2753e92b249668d8389db72282cabf048aea34c3");

        let mut commit_with_content_reordered = Object::Commit {
            path: PathBuf::from("/tmp"),
            content: vec![
                String::from("blob\te28de939637c3a53b10c53ebfc3203acbf168717\t.rustfmt.toml"),
                String::from("blob\t9c00c0236a2b133dbfbc7ad799f9ea5ce685c2e4\tTODO.md"),
                String::from("blob\t77c57febfc94ff583a1a15a004d01cf6e16a4442\tCargo.toml"),
                String::from("tree\t17d520fea68d0d107a4e8becad26e47f37e73aab\tsrc"),
                String::from("blob\t32bab984c61ba43ba15e479b23df5e828aa43864\tCargo.lock"),
                String::from("tree\t9dffa2d73d8b2f67a59768600023cd32b21ba7ac\tdummy_app"),
            ],
            properties: commit_properties.clone(),
            message: String::from(message),
            timestamp: Duration::new(1680961369, 0),
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
                String::from("blob\t0a883d942f72a18558810edd255d846f408ed35a\tmain.rs"),
                String::from("blob\t44dd4de05dddc1235fdb19bf1ab2dc4c11178da8\tobject.rs"),
                String::from("blob\t5651a0070d5d6031b7b9e53e7d962acfb9fdfba1\tworktree.rs"),
                String::from("blob\td4c47993f35fec5888d41e74aa67cda00242376a\tlib.rs"),
                String::from("blob\tf8cf364dd0b44a6c44c99600ff6d2ca9111c3a23\terror.rs"),
            ],
            digest: String::default(),
        };

        assert!(tree.update_digest().is_ok());
        assert!(tree.digest() == "1bba1312886239216792daa5a21d9ecf65cebd75");

        let mut tree_with_content_reordered = Object::Tree {
            path: PathBuf::from("/tmp"),
            content: vec![
                String::from("blob\td4c47993f35fec5888d41e74aa67cda00242376a\tlib.rs"),
                String::from("blob\t44dd4de05dddc1235fdb19bf1ab2dc4c11178da8\tobject.rs"),
                String::from("blob\t5651a0070d5d6031b7b9e53e7d962acfb9fdfba1\tworktree.rs"),
                String::from("blob\tf8cf364dd0b44a6c44c99600ff6d2ca9111c3a23\terror.rs"),
                String::from("blob\t0a883d942f72a18558810edd255d846f408ed35a\tmain.rs"),
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
        assert!(content_line.unwrap() == String::from("tree\tdigest\ttmp"));
    }

    #[test]
    fn blob_obj_content_line() {
        let blob = Object::Blob {
            path: PathBuf::from("/tmp/odyssey.txt"),
            full_path: PathBuf::default(),
            content: String::default(),
            digest: String::from("digest"),
        };

        let content_line = blob.obj_content_line();
        assert!(content_line.is_ok());
        assert!(content_line.unwrap() == String::from("blob\tdigest\todyssey.txt"));
    }
}
