#[macro_use]
extern crate serde_derive;

use clap::{Arg, Command};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

macro_rules! die(
  ($($arg:tt)*) => { {
    writeln!(std::io::stderr(), $($arg)*)
      .expect("Failed to print to stderr");
    std::process::exit(1);
  } }
);

#[derive(Deserialize)]
struct Config {
    commands: BTreeMap<String, String>,
    user_commands: Option<BTreeMap<String, BTreeMap<String, String>>>,
}

fn read_config<P: AsRef<std::path::Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let mut contents = String::new();

    File::open(path)?.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

fn run_command(command: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let mut child = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .spawn()?;

    child
        .wait()?
        .code()
        .ok_or_else(|| std::io::Error::last_os_error().into())
}

fn main() {
    let matches = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("resh is a restricted (ssh) shell that only allows whitelisted commands")
        .arg(
            Arg::new("command")
                .short('c')
                .help("Alias of command to execute")
                .value_name("COMMAND"),
        )
        .get_matches();

    let command_alias = match matches.get_one::<String>("command") {
        Some(cmd) => cmd.clone(),

        None => match std::env::var("SSH_ORIGINAL_COMMAND") {
            Ok(cmd) => cmd,
            _ => die!("Usage: {} <command alias>", clap::crate_name!()),
        },
    };

    let config_file = std::env::var("RESH_CONFIG").unwrap_or_else(|_| "/etc/resh.toml".to_string());

    let config: Config = read_config(&config_file).unwrap_or_else(|e| {
        die!("Failed to read {}: {}", config_file, e);
    });

    let username = std::env::var("USER").unwrap_or_else(|_| "default".to_string());

    let commands = config
        .user_commands
        .as_ref()
        .and_then(|user_cmds| user_cmds.get(&username))
        .unwrap_or_else(|| &config.commands);

    let full_command = match commands.get(&command_alias) {
        Some(cmd) => cmd,
        None => die!("Undefined command alias: {}", command_alias),
    };

    let exitcode = run_command(full_command).unwrap_or(1);

    std::process::exit(exitcode);
}
