
use clap::{Command, Arg};

use commands::get::get;

fn main() {

    let command = Command::new("netplancli")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("get")
                .about("Get all the things \\o/")
                .arg(Arg::new("key").default_value("all"))
        );
    
    match command.get_matches().subcommand() {
        Some(("get", args)) => {
            let key =  args.get_one::<String>("key").unwrap();
            get(key);
        },
        _ => {}
    }

}
