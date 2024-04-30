#[macro_use]
extern crate serde_derive;

use clap::{crate_authors, crate_name, crate_version, Arg, Command};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    io::{self, prelude::*},
    process::Command as ProcessCommand,
};

macro_rules! die(
    ($($arg:tt)*) => { {
        writeln!(io::stderr(), $($arg)*).expect("Failed to print to stderr");
        std::process::exit(1);
    } }
);

#[derive(Deserialize)]
struct Config {
    commands: BTreeMap<String, String>,
    user_commands: Option<BTreeMap<String, BTreeMap<String, String>>>,
}

fn read_config<P: AsRef<std::path::Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    toml::from_str(&contents).map_err(|e| e.into())
}

fn run_command(command: &str, args: &str) -> Result<i32, Box<dyn Error>> {
    let formatted_command = args
        .split_whitespace()
        .enumerate()
        .fold(command.replace("%@", args), |cmd, (i, arg)| {
            cmd.replace(&format!("%{}", i + 1), arg)
        });

    ProcessCommand::new("/bin/sh")
        .arg("-c")
        .arg(&formatted_command)
        .spawn()?
        .wait()?
        .code()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "No exit code available").into()
        })
}

fn main() {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("resh is a restricted shell that only allows whitelisted commands")
        .arg(
            Arg::new("command")
                .short('c')
                .help("Alias of command to execute")
                .value_name("COMMAND"),
        )
        .get_matches();

    let command_alias_and_args = std::env::var("SSH_ORIGINAL_COMMAND")
        .or_else(|_| {
            matches
                .get_one::<String>("command")
                .cloned()
                .ok_or_else(|| "Command not found")
        })
        .unwrap_or_else(|_| die!("Usage: {} <command alias> <arguments>", crate_name!()));

    let mut command_args = command_alias_and_args.split_whitespace();
    let command_alias = command_args
        .next()
        .unwrap_or_else(|| die!("Usage: {} <command alias> <arguments>", crate_name!()));

    let config_file = std::env::var("RESH_CONFIG").unwrap_or_else(|_| "/etc/resh.toml".to_string());
    let config =
        read_config(&config_file).unwrap_or_else(|e| die!("Failed to read {}: {}", config_file, e));

    let username = std::env::var("USER").unwrap_or_else(|_| "default".to_string());
    let command = config
        .user_commands
        .as_ref()
        .and_then(|user_cmds| {
            user_cmds
                .get(&username)
                .and_then(|cmds| cmds.get(command_alias))
        })
        .or_else(|| config.commands.get(command_alias))
        .unwrap_or_else(|| die!("Undefined command alias: {}", command_alias));

    let exitcode =
        run_command(command, &command_args.collect::<Vec<&str>>().join(" ")).unwrap_or(1);
    std::process::exit(exitcode);
}
