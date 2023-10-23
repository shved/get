pub mod error;
mod object;
mod paths;
mod worktree;

use crate::error::Error;
use crate::worktree::RepoWithState;

use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use log::warn;
use serde::Deserialize;
use toml;
use users::get_current_username;

const DEFAULT_FILE_PERMISSIONS: u32 = 0o644;
const DEFAULT_DIR_PERMISSIONS: u32 = 0o755;
const EMPTY_REF: &str = "0000000000000000000000000000000000000000";
const DEFAULT_IGNORE: &[&str] = &[".get", ".get.toml"]; // Default ignore patterns.

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    ignore: Vec<String>,
    #[serde(default = "default_author")]
    author: String,
    // #[serde(default)]
    // remotes: Vec<String>, // TODO Use some URL kind of type.
}

#[derive(Debug, Clone)]
pub struct Repo {
    work_dir: PathBuf,
    config: Config,
    head: String,
}

impl Repo {
    // TODO Rework it to actually take &Path instead of mut PathBuf.
    pub fn init(cur_dir: &mut PathBuf) -> Result<Repo, Error> {
        paths::check_no_repo_dir(cur_dir)?;

        create_utility_dirs(cur_dir)?;
        create_utility_files(cur_dir)?;

        let config = resolve_config(cur_dir.as_path())?;

        Ok(Repo {
            work_dir: cur_dir.clone(),
            config,
            head: String::from(EMPTY_REF),
        })
    }

    pub fn try_from(cur_dir: &PathBuf) -> Result<Repo, Error> {
        let work_dir = paths::repo_dir(cur_dir)?;
        let config = resolve_config(cur_dir.as_path())?;
        let head = read_head(work_dir.as_path())?;

        Ok(Repo {
            work_dir,
            config,
            head,
        })
    }

    pub fn commit(&self, msg: Option<&str>, now: SystemTime) -> Result<String, Error> {
        // TODO Change default message to smthg more informative.
        let message = msg.unwrap_or("default commit message");

        let repo_with_state = RepoWithState::from_files(self.clone(), message, now)?;
        let new_commit_digest = repo_with_state.save_commit().map(|s| s.to_string())?;
        self.write_head(new_commit_digest.as_str())?;

        Ok(new_commit_digest)
    }

    pub fn restore(&self, digest: &str) -> Result<(), Error> {
        // Check the commit exists before cleaning the directory.
        let _ = self.read_commit_object(digest.to_owned())?;

        worktree::clean_before_restore(self.work_dir.as_path(), &self)?;
        let repo_with_state = RepoWithState::from_commit(self.clone(), digest.to_owned())?;
        repo_with_state.restore_files()?;
        self.write_head(digest)?;

        Ok(())
    }

    fn write_head(&self, digest: &str) -> Result<(), Error> {
        fs::write(paths::head_path(self.work_dir.as_ref()), digest)?;

        Ok(())
    }
}

fn resolve_config(work_dir: &Path) -> Result<Config, Error> {
    let config: Config;

    if let Ok(cfg_file) = fs::read_to_string(work_dir.join(".get.toml")) {
        config = toml::from_str(cfg_file.as_ref()).map_err(|_| Error::Unexpected)?;
    } else {
        warn!("could not read config file, default is set");
        config = default_config();
    }

    Ok(config)
}

fn read_head(base_path: &Path) -> Result<String, Error> {
    let str = fs::read_to_string(paths::head_path(base_path))?;

    Ok(str)
}

fn create_utility_dirs(cur_path: &mut PathBuf) -> Result<(), Error> {
    // Crete `.get`.
    cur_path.push(paths::REPO_DIR);
    create_dir(cur_path)?;

    // Crete `.get/objects`.
    cur_path.push(paths::OBJECTS_DIR);
    create_dir(cur_path)?;

    // Crete `.get/objects/*` dirs.
    cur_path.push(paths::COMMITS_DIR);
    create_dir(cur_path)?;
    cur_path.pop();

    cur_path.push(paths::TREE_DIR);
    create_dir(cur_path)?;
    cur_path.pop();

    cur_path.push(paths::BLOB_DIR);
    create_dir(cur_path)?;
    cur_path.pop();

    cur_path.pop();
    cur_path.pop();

    Ok(())
}

fn create_dir(cur_path: &mut PathBuf) -> io::Result<()> {
    fs::create_dir(cur_path.as_path())?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_DIR_PERMISSIONS),
    )?;

    Ok(())
}

fn create_utility_files(cur_path: &mut PathBuf) -> io::Result<()> {
    cur_path.push(paths::REPO_DIR);
    cur_path.push(paths::HEAD_FILE);
    fs::write(cur_path.as_path(), EMPTY_REF)?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_FILE_PERMISSIONS),
    )?;
    cur_path.pop();

    cur_path.push(paths::LOG_FILE);
    fs::File::create(cur_path.as_path())?;
    fs::set_permissions(
        cur_path.as_path(),
        fs::Permissions::from_mode(DEFAULT_FILE_PERMISSIONS),
    )?;
    cur_path.pop();

    cur_path.pop();

    Ok(())
}

fn default_config() -> Config {
    Config {
        ignore: vec![],
        author: default_author(),
    }
}

fn default_author() -> String {
    get_current_username()
        .unwrap_or_else(|| {
            warn!("couldn't fetch user name, default user name used instead");
            OsString::from("unknown author")
        })
        .into_string()
        .unwrap_or(String::from("unknown author"))
}
