mod get;

use clap::{arg, Command};
use std::env;

fn main() {
    let matches = Command::new("get")
        .version("1.0")
        .author("Vitalii Shvedchenko <vitaly.shvedchenko@gmail.com>")
        .about("Like git but worse")
        .subcommand_required(true)
        .subcommand(Command::new("init").about("creates new repo in currenct directory"))
        .subcommand(Command::new("commit")
                    .about("saves the changes")
                    .arg(arg!([message] "optional message")))
        .get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            get::init(env::current_dir().expect("can't get current dir path"));
        }
        Some(("commit", sub_matches)) => {
            let msg = sub_matches.get_one::<String>("message");
            get::commit(msg);
        },
        _ => unreachable!("unknown command"),
    }
}
