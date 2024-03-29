use get::error::Error;

use std::env;
use std::process::exit;
use std::time::SystemTime;

use clap::{arg, Command};
use log::{error, info};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::max())
        .init();

    let matches = Command::new("get")
        .version("1.0")
        .author("Vitalii Shvedchenko <vitaly.shvedchenko@gmail.com>")
        .about("Like git but worse")
        .subcommand_required(true)
        .subcommand(Command::new("init").about("creates new repo in currenct directory"))
        .subcommand(
            Command::new("commit")
                .about("saves the changes")
                .arg(arg!([message] "optional message")),
        )
        .subcommand(
            Command::new("restore")
                .about("resotres saved files")
                .arg(arg!([digest] "commit digest to restore").required(true)),
        )
        .get_matches();

    let mut cur_dir = env::current_dir().unwrap_or_else(|e| {
        error!("{}", Error::IoError(e));
        exit(1);
    });

    match matches.subcommand() {
        Some(("init", _)) => match get::Repo::init(&mut cur_dir) {
            Ok(_) => {
                info!("Repo created!");
            }
            Err(err) => {
                error!("{err}");
                exit(1);
            }
        },
        Some(("commit", sub_matches)) => {
            let msg = sub_matches.get_one::<String>("message");
            let sys_time = SystemTime::now();
            let repo = match get::Repo::try_from(&cur_dir) {
                Ok(repo) => repo,
                Err(err) => {
                    error!("{err}");
                    exit(1);
                }
            };
            match repo.commit(msg.map(|s| s.as_str()), sys_time) {
                Ok(commit_digest) => {
                    info!("Commit {} saved successfully.", commit_digest);
                }
                Err(err) => {
                    error!("{err}");
                    exit(1);
                }
            }
        }
        Some(("restore", sub_matches)) => {
            // We unwrap here safely since digest is explicitly required by clap.
            let digest = sub_matches.get_one::<String>("digest").unwrap();
            let repo = match get::Repo::try_from(&cur_dir) {
                Ok(repo) => repo,
                Err(err) => {
                    error!("{err}");
                    exit(1);
                }
            };
            match repo.restore(digest.as_str()) {
                Ok(_) => {
                    info!("Commit {} restored successfully.", digest);
                }
                Err(err) => {
                    error!("{err}");
                    exit(1);
                }
            }
        }
        _ => unreachable!("get: unknown subcommand"),
    }
}
