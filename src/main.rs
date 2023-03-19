use clap::{arg, Command};

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
            // if sub_matches
            println!("repo initialized")
        }
        Some(("commit", sub_matches)) => {
            let msg = sub_matches.get_one::<String>("message");
            match msg {
                Some(message) => println!("{}", message),
                None => println!("default commit message"),
            }
        },
        _ => unreachable!("unknown command"),
    }
}
