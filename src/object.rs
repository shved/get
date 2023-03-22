use std::path::PathBuf;

#[allow(dead_code)]
enum ObjType {
    Commit,
    Tree,
    Blob,
}

#[allow(dead_code)]
pub(crate) struct Object {
    obj_type: ObjType,
    parent: Option<Box<Object>>,
    children: Vec<Box<Object>>,
    path: PathBuf,
    parent_commit_digest: [u8; 20],
    //
    // Name             string
    // ParentPath       string
    // Path             string
    // ParentCommitHash string
    // CommitMessage    string
    // HashString       string
    // Timestamp        time.Time

    // sha          []byte
    // contentLines []string
    // gzipContent  string
}
