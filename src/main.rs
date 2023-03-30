use clap::{arg, Command};
use std::env;

fn main() {
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
        Some(("init", _)) => {
            get::init(&mut root_path);
        }
        Some(("commit", sub_matches)) => {
            let msg = sub_matches.get_one::<String>("message");
            get::commit(root_path, msg.map(|s| s.as_str()));
        }
        _ => unreachable!("get: unknown subcommand"),
    }
}
