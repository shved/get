use clap::{arg, Command};
use std::env;
use std::process::exit;

use log::{error, info};

fn main() {
    let _ = env_logger::builder()
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
        .get_matches();

    let mut root_path = env::current_dir().expect("get: can't get current dir path");

    match matches.subcommand() {
        Some(("init", _)) => match get::init(&mut root_path) {
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
            match get::commit(root_path, msg.map(|s| s.as_str())) {
                Ok(commit_digest) => {
                    info!("Commit {} saved successfully.", commit_digest)
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
