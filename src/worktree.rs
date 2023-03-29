use crate::object::Object;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// TODO rework it to be a single field struct or just a commit itself cause the root of this graph
// is always a commit.
struct Worktree {
    graph: Option<Box<Object>>,
}

impl Worktree {
    fn fs_snapshot(
        p: &mut PathBuf,
        message: &str,
        ignore: &[&str],
        timestamp: SystemTime,
    ) -> Worktree {
        let repo_root = p.as_path();

        let parent_commit_hash = read_head(repo_root);

        let commit = Box::new(Object::Commit {
            children: Vec::default(),
            parent_commit_digest: parent_commit_hash,
            commit_digest: String::default(),
            commit_msg: message.to_string(),
            timestamp: timestamp,
            path: repo_root.to_path_buf(),
        });

        let wt = Worktree {
            graph: Some(commit),
        };

        for entry in fs::read_dir(repo_root)
            .expect("get: can't read dir")
            .into_iter()
            .filter(|e| !is_ignored(e.as_ref().unwrap().path(), repo_root, ignore))
        {
            dbg!(entry.unwrap());
        }

        wt
    }
}

// fn index_dir(obj: Object) {
//     match obj {
//         Commit(commit) => println!("asdf"),
//         Tree => {
//             for entry in fs::read_dir(obj.path)
//                 .expect("get: can't read dir")
//                 .into_iter()
//                 .filter(|e| !is_ignored(e.as_ref().unwrap().path(), repo_root, ignore))
//             {
//                 dbg!(entry.unwrap());
//             }
//         }
//     }
// }

pub(crate) fn commit(
    cur_path: &mut PathBuf,
    message: &str,
    ignore: &[&str],
    timestamp: SystemTime,
) {
    let _ = Worktree::fs_snapshot(cur_path, message, ignore, timestamp);
}

fn is_ignored(e: PathBuf, repo_root: &Path, ignored: &[&str]) -> bool {
    // TODO handle None from repo_root.parent()
    for i in ignored.iter() {
        if e.strip_prefix(repo_root.parent().unwrap())
            .unwrap()
            .to_str()
            .unwrap()
            .contains(i)
        {
            return true;
        }
    }

    false
}

fn read_head(repo_root: &Path) -> String {
    // TODO use static path names
    fs::read_to_string(repo_root.join(super::REPO_DIR).join(super::HEAD_FILE))
        .expect("get: cant read HEAD")
        .trim()
        .into()
}
