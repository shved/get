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

    match matches.subcommand() {
        Some(("init", _)) => {
            let mut root_path = env::current_dir().expect("get: can't get current dir path");
            get::init(&mut root_path);
        }
        Some(("commit", sub_matches)) => {
            let msg = sub_matches.get_one::<String>("message");
            // TODO make it support getignore file with proper ingore patterns
            // and store the slice in static segment.
            let getignore = &[".git", ".gitignore", "target", ".get", ".getignore"];
            get::commit(msg.map(|s| s.as_str()), getignore);
        }
        _ => unreachable!("get: unknown subcommand"),
    }
}
