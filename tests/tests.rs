use std::fs;
// use std::mem;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use tempdir::TempDir;

const FIRST_COMMIT_DIGEST: &str = "aa3392b46447d7e2023e2706929f97b4eb7271fc";

#[test]
fn repo_workflow() {
    let repo_root = TempDir::new("get_app_test").unwrap();
    let mut root_path = repo_root.path().to_owned();

    setup_project_dir(&mut root_path);

    assert!(get::commit(root_path.clone(), None, SystemTime::now()).is_err());

    // Init.
    assert!(get::init(&mut root_path).is_ok());

    assert!(root_path.as_path().join(".get").is_dir());
    assert!(root_path.as_path().join(".get/objects").is_dir());
    assert!(root_path.as_path().join(".get/objects/commit").is_dir());
    assert!(root_path.as_path().join(".get/objects/tree").is_dir());
    assert!(root_path.as_path().join(".get/objects/blob").is_dir());
    assert!(root_path.as_path().join(".get/HEAD").is_file());
    assert!(root_path.as_path().join(".get/LOG").is_file());

    // Initial commit.
    let commit_message: Option<&str> = Some("descriptive message");

    let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1680961369);

    let first_commit_digest = get::commit(root_path.clone(), commit_message, timestamp);

    assert!(first_commit_digest.is_ok());
    assert_eq!(first_commit_digest.unwrap(), FIRST_COMMIT_DIGEST,);

    let cur_head = fs::read_to_string(repo_root.path().join(".get/HEAD"));
    assert!(cur_head.is_ok());
    assert_eq!(cur_head.unwrap(), FIRST_COMMIT_DIGEST,);

    // Init again and fail.
    assert!(get::init(&mut root_path).is_err());

    // Uncomment to review the project folder after test run.
    // mem::forget(repo_root);
}

fn setup_project_dir(root_path: &mut PathBuf) {
    root_path.push("test");
    fs::create_dir(root_path.as_path()).unwrap();

    root_path.push("test_file1.txt");
    fs::write(root_path.as_path(), b"dukkha (literally \"suffering\"; here \"unsatisfactoriness\") is an innate characteristic of existence in the realm of samsara;\n").unwrap();
    root_path.pop();

    root_path.push("test_file2.txt");
    fs::write(root_path.as_path(), b"samudaya (origin, arising, combination; 'cause'): dukkha arises or continues with tanha (\"craving, desire or attachment, lit. \"thirst\"). While tanha is traditionally interpreted in western languages as the 'cause' of dukkha, tanha can also be seen as the factor tying us to dukkha, or as a response to dukkha, trying to escape it;\n").unwrap();
    root_path.pop();

    root_path.push("test_inside_test");
    fs::create_dir(root_path.as_path()).unwrap();

    root_path.push("test_file3.txt");
    fs::write(root_path.as_path(), b"nirodha (cessation, ending, confinement): dukkha can be ended or contained by the renouncement or letting go of this tanha; the confinement of tanha releases the excessive bind of dukkha;\n").unwrap();
    root_path.pop();

    root_path.push("test_file4.txt");
    fs::write(root_path.as_path(), b"marga (path, Noble Eightfold Path) is the path leading to the confinement of tanha and dukkha.").unwrap();
    root_path.pop();

    // root_path.push("empty_dir");
    // fs::create_dir(root_path.as_path()).unwrap();
    // root_path.push(".getkeep");
    // File::create(root_path.as_path()).unwrap();
    // root_path.pop();
    // root_path.pop();

    root_path.pop();
    root_path.pop();

    root_path.push("test_file.txt");
    fs::write(root_path.as_path(), b"thats\nall,\nfolks!").unwrap();
    root_path.pop();
}
