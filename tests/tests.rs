use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use pretty_assertions::assert_eq;
use tempdir::TempDir;
use walkdir::WalkDir;

const FIRST_COMMIT_DIGEST: &str = "da72e4ac4723a3a37697f9027b653804d74303af";
const SECOND_COMMIT_DIGEST: &str = "669ec3ec589015496a5bdae6e48a7508f7392027";

#[test]
fn repo_workflow() {
    // TODO add the whole directory content digest calculation to detech any content changes after
    // restoring a commit.
    let repo_root = TempDir::new("get_app_test").unwrap();
    let mut working_dir = repo_root.path().to_owned();

    setup_project_dir(&mut working_dir);

    assert!(get::commit(working_dir.clone(), None, SystemTime::now()).is_err());

    // Init.
    assert!(get::init(&mut working_dir).is_ok());

    assert!(working_dir.as_path().join(".get").is_dir());
    assert!(working_dir.as_path().join(".get/objects").is_dir());
    assert!(working_dir.as_path().join(".get/objects/commit").is_dir());
    assert!(working_dir.as_path().join(".get/objects/tree").is_dir());
    assert!(working_dir.as_path().join(".get/objects/blob").is_dir());
    assert!(working_dir.as_path().join(".get/HEAD").is_file());
    assert!(working_dir.as_path().join(".get/LOG").is_file());

    // Initial commit.
    let commit_message: Option<&str> = Some("descriptive message");
    let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1680961369);
    let first_commit_digest = get::commit(working_dir.clone(), commit_message, timestamp);

    assert!(first_commit_digest.is_ok());
    assert_eq!(first_commit_digest.unwrap(), FIRST_COMMIT_DIGEST);

    let cur_head = fs::read_to_string(repo_root.path().join(".get/HEAD"));
    assert!(cur_head.is_ok());
    assert_eq!(cur_head.unwrap(), FIRST_COMMIT_DIGEST,);

    // Init again and fail.
    assert!(get::init(&mut working_dir).is_err());

    let after_initial_commit = working_files_snapshot(&working_dir);

    // Modify tree and make new commit.
    modify_files(&working_dir.clone());

    let after_changes = working_files_snapshot(&working_dir);
    let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1680961869);
    let commit_message: Option<&str> = Some("second commit descriptive message");
    let second_commit_digest = get::commit(working_dir.clone(), commit_message, timestamp);

    assert!(second_commit_digest.is_ok());
    assert_eq!(second_commit_digest.unwrap(), SECOND_COMMIT_DIGEST);

    // Check commit digest was written to HEAD.
    let cur_head = fs::read_to_string(repo_root.path().join(".get/HEAD"));
    assert!(cur_head.is_ok());
    assert_eq!(cur_head.unwrap(), SECOND_COMMIT_DIGEST,);

    assert!(get::restore(working_dir.clone(), FIRST_COMMIT_DIGEST).is_ok());

    // Check commit digest was updated into HEAD after restore a previous commit.
    let cur_head = fs::read_to_string(repo_root.path().join(".get/HEAD"));
    assert!(cur_head.is_ok());
    assert_eq!(cur_head.unwrap(), FIRST_COMMIT_DIGEST,);

    let after_restore_init_commit = working_files_snapshot(&working_dir);

    assert_eq!(after_initial_commit, after_restore_init_commit);

    assert!(get::restore(working_dir.clone(), SECOND_COMMIT_DIGEST).is_ok());
    let after_restore_second_commit = working_files_snapshot(&working_dir);

    assert_eq!(after_changes, after_restore_second_commit);

    // Check commit digest was updated into HEAD after restore a previous commit.
    let cur_head = fs::read_to_string(repo_root.path().join(".get/HEAD"));
    assert!(cur_head.is_ok());
    assert_eq!(cur_head.unwrap(), SECOND_COMMIT_DIGEST,);

    // std::mem::forget(repo_root);
}

fn modify_files(working_dir: &PathBuf) {
    fs::write(
        working_dir.join("test_file.txt"),
        b"and now it is modified!",
    )
    .unwrap();

    fs::rename(
        working_dir.join("testdir").join("test_file1.txt"),
        working_dir.join("testdir").join("new_name.txt"),
    )
    .unwrap();

    fs::write(
        working_dir.join("testdir").join("new_name.txt"),
        b"and now it is modified!",
    )
    .unwrap();
}

fn setup_project_dir(working_dir: &mut PathBuf) {
    working_dir.push("testdir");
    fs::create_dir(working_dir.as_path()).unwrap();

    working_dir.push("test_file1.txt");
    fs::write(working_dir.as_path(), b"dukkha (literally \"suffering\"; here \"unsatisfactoriness\") is an innate characteristic of existence in the realm of samsara;\n").unwrap();
    working_dir.pop();

    working_dir.push("test_file2.txt");
    fs::write(working_dir.as_path(), b"samudaya (origin, arising, combination; 'cause'): dukkha arises or continues with tanha (\"craving, desire or attachment, lit. \"thirst\"). While tanha is traditionally interpreted in western languages as the 'cause' of dukkha, tanha can also be seen as the factor tying us to dukkha, or as a response to dukkha, trying to escape it;\n").unwrap();
    working_dir.pop();

    working_dir.push("nested");
    fs::create_dir(working_dir.as_path()).unwrap();

    working_dir.push("test_file3.txt");
    fs::write(working_dir.as_path(), b"nirodha (cessation, ending, confinement): dukkha can be ended or contained by the renouncement or letting go of this tanha; the confinement of tanha releases the excessive bind of dukkha;\n").unwrap();
    working_dir.pop();

    working_dir.push("test_file4.txt");
    fs::write(working_dir.as_path(), b"marga (path, Noble Eightfold Path) is the path leading to the confinement of tanha and dukkha.").unwrap();
    working_dir.pop();

    // working_dir.push("empty_dir");
    // fs::create_dir(working_dir.as_path()).unwrap();
    // working_dir.push(".getkeep");
    // File::create(working_dir.as_path()).unwrap();
    // working_dir.pop();
    // working_dir.pop();

    working_dir.pop();
    working_dir.pop();

    working_dir.push("test_file.txt");
    fs::write(working_dir.as_path(), b"thats\nall,\nfolks!").unwrap();
    working_dir.pop();
}

fn working_files_snapshot(p: &Path) -> String {
    let mut paths: Vec<String> = Vec::new();

    for entry in WalkDir::new(p) {
        let path_string = entry.unwrap().path().to_str().unwrap().to_owned();
        if path_string.contains(".get") {
            continue;
        }

        let path = path_string.to_owned();
        paths.push(path);
    }

    paths.sort();
    let mut ret = paths.join("\n");
    ret.push('\n');

    ret
}
